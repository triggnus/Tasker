//! # Tasker
//! Interactive command line tool for creating and tracking tasks. Tasks have a name, an optional due date and optional notes.
//!
//! Author: Rob Teeple <somethingobscure@gmail.com>
//!
//! This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as
//! published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.
//!
//! This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty
//! of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more de
//!
//! You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.

use crate::confirmation::ConfirmationDialog;
use crate::popup::{Popup, centered_rect};
use crate::task::Task;
use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::layout::Position;
use ratatui::style::{Color, Style, Stylize};
use ratatui::text::Line;
use ratatui::{
	Terminal,
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout},
	widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use rusqlite::Connection;
use std::io;

mod char_to_vec;
mod insert_command;
mod confirmation;
mod popup;
mod task;
mod delimiter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
	// --- DB Initialization ---
	let conn = Connection::open("task_db.sqlite")?;

	// If the db does not exist, create it.
	conn.execute(
		"CREATE TABLE IF NOT EXISTS tasks (
			id INTEGER PRIMARY KEY,
			description TEXT NOT NULL,
			notes TEXT NULL,
			due_date DATE NULL
		)",
		(),
	)?;

	// Load the database in to a Vec<Task>
	let mut tasks = conn
		.prepare("SELECT id, description, notes, due_date FROM tasks ORDER BY id ASC")?
		.query_map([], |row| {
			Ok(Task::new(
				row.get("id")?,
				row.get("description")?,
				row.get("notes")?,
				row.get("due_date")?,
			))
		})?
		.map(|task| task.unwrap())
		.collect::<Vec<Task>>();

	// --- Terminal Setup ---
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// --- Application State ---
	let mut input = String::new();
	let mut show_task_popup = false;
	let mut show_help = false;
	let mut command = String::new();
	let mut task_popup_command = String::new();
	let mut delete_confirm_dialog = ConfirmationDialog::new();

	// --- UI Main Loop ---
	loop {
		terminal.draw(|f| {
			let size = f.area();
			let chunks = Layout::default()
				.direction(Direction::Vertical)
				.margin(2)
				.constraints([
					Constraint::Length(3), // Input area
					Constraint::Min(1),    // List area
				])
				.split(f.area());

			// Render Input Box
			let input_widget = Paragraph::new(Line::from(vec!["# ".into(), input.clone().into()]))
				.block(
					Block::default()
						.borders(Borders::ALL)
						.title(Line::from(vec![
							" Enter Command (Enter)".into(),
							" ['help' for options] ".dark_gray(),
						])),
				);
			f.render_widget(input_widget, chunks[0]);

			// Render List
			let list_items: Vec<ListItem> = tasks
				.iter()
				.map(|task| ListItem::new(task.to_string()))
				.collect();

			let list_widget = List::new(list_items).block(
				Block::default()
					.borders(Borders::ALL)
					.title(" Current Tasks (Esc to Quit) ")
					.style(Style::new().bg(Color::DarkGray).fg(Color::White)),
			);
			f.render_widget(list_widget, chunks[1]);

			// --- Task Popup Dialog ---
			if show_task_popup {
				let area = centered_rect(40, 20, size); // Dialog size: 60% width, 20% height
				f.render_widget(Clear, area); // This clears the background under the popup

				// internally, the command is all lowercase. This code capitalizes the first letter.
				// Rust has no convenient methods for this, for some reason.
				let title = format!(
					"{}{} Task",
					&command[..1].to_string().to_uppercase(),
					&command[1..]
				);

				let command_color = if command == "add" {
					Color::Cyan
				} else {
					Color::Red
				};

				let popup = Popup::default()
					.style(Style::new().yellow())
					.title(title)
					.content(task_popup_command.as_str())
					.title_style(Style::new().white().bold())
					.border_style(Style::new().fg(command_color));

				// Calculate the position of the cursor
				let mut cur_pos = area.as_position();
				cur_pos.x = cur_pos.x + task_popup_command.len() as u16 + 1;
				cur_pos.y += 1;

				f.set_cursor_position(cur_pos);

				f.render_widget(popup, area);
			} else if show_help {
				let area = centered_rect(70, 60, size);
				f.render_widget(Clear, area);

				let popup = Popup::default()
					.title(Line::from(" Tasker Usage ").white())
					.content(HELP_STR);

				f.render_widget(popup, area);
			}

			// --- Confirmation Dialog ---
			if delete_confirm_dialog.is_shown() {
				let area = centered_rect(40, 20, size);
				f.render_widget(Clear, area);

				let popup = Popup::default()
					.style(Style::new().yellow())
					.title(delete_confirm_dialog.title())
					.content(delete_confirm_dialog.text())
					.centered(true)
					.title_style(Style::new().yellow().bold())
					.border_style(Style::new().red().bold());

				f.render_widget(popup, area);
			}

			// Draw the cursor in the right place
			if !show_task_popup && !delete_confirm_dialog.is_shown() {
				f.set_cursor_position(Position::new(
					// chunks[0] is the left most side of the input box
					// Add the length of the input string + 1
					chunks[0].x + input.len() as u16 + 3,
					// Drop the row by 1
					chunks[0].y + 1,
				));
			}
		})?;

		// --- INPUT HANDLING ---
		if let Event::Key(key) = event::read()? {
			match key.code {
				KeyCode::Esc => {
					if show_task_popup {
						// hide task popup
						show_task_popup = false;
					} else if delete_confirm_dialog.is_shown() {
						// hide delete confirm dialog
						delete_confirm_dialog.hide();
					} else if show_help {
						// hide help dialog
						show_help = false;
					} else {
						// close the program
						break;
					}
				}
				KeyCode::Enter => {
					if show_help {
						// hide the help dialog
						show_help = false;
					}

					if !input.is_empty() {
						// NOTE: This is a bit convoluted, but it works. Need to rethink this.
						match input.to_lowercase().as_str() {
							"a" | "add" => {
								show_task_popup = true;
								command = "add".to_string();
							}
							"d" | "delete" => {
								show_task_popup = true;
								command = "delete".to_string();
							}
							"h" | "help" => show_help = true,
							"exit" | "quit" | "q" => break,
							_ => {
								// --- Inline command support ---
								if input.starts_with("d")
									&& let Some(sub_commands) = input.split_once(" ")
								{
									let tasks_to_delete = sub_commands.1.to_string();

									delete_confirm_dialog
										.set_tasks_to_delete(tasks_to_delete.clone());
									delete_confirm_dialog.show();
								}

								if input.starts_with("a")
									&& let Some(sub_commands) = input.split_once(" ")
								{
									tasks.push(Task::insert_task(
										&conn,
										sub_commands.1.to_string(),
									)?);
								}
							}
						}

						input.clear();
					}

					// if the add/delete task popup is shown and the command entered isn't empty
					if show_task_popup && !task_popup_command.is_empty() {
						if command == "add" || command == "a" {
							let new_task = Task::insert_task(&conn, task_popup_command.clone())?;
							tasks.push(new_task);
						} else if command.starts_with("delete")
							|| command.chars().nth(1).unwrap() == 'd'
						{
							delete_confirm_dialog.set_tasks_to_delete(task_popup_command.clone());
							delete_confirm_dialog.show();
						}

						task_popup_command.clear();
						show_task_popup = false;
					}
				}
				KeyCode::Char(c) => {
					// c is every non-command character typed while the program is running.

					if show_task_popup {
						// if the popup is rendered, then pipe the output to task_popup_command instead of input.
						task_popup_command.push(c);
					} else if delete_confirm_dialog.is_shown() {
						// else if the confirmation dialog is rendered, capture the 'y' char if pressed to confirm task deletion
						if c == 'y' {
							Task::remove_tasks(
								&conn,
								&mut tasks,
								delete_confirm_dialog.tasks_to_delete(),
							)?;
						}
						delete_confirm_dialog.reset();
					} else if show_help {
						// if the help window is being shown, do nothing.
						continue;
					} else {
						// Base case. Pipe input to the input variable.
						input.push(c);
					}
				}
				KeyCode::Backspace => {
					// if the popup is rendered, backspace should operate on task_popup_command
					if show_task_popup {
						task_popup_command.pop();
					}
					// Base case. Backspace removes the last char of the input string.
					else {
						input.pop();
					}
				}
				// No support for any other class of key presses (function keys, arrows, tab, etc.)
				_ => {}
			}
		}
	}

	// --- CLEANUP ---
	disable_raw_mode()?;
	execute!(
		terminal.backend_mut(),
		LeaveAlternateScreen,
		DisableMouseCapture
	)?;
	terminal.show_cursor()?;

	Ok(())
}

// text displayed when the help option is selected
const HELP_STR: &str = "Options:
    add    | a        : Add a task
    delete | d        : Delete a task
    help   | h        : Show this help
    quit   | q | exit : Quit Tasker

Add a new task:
    # add 'task name' 'due in days' [notes]
    'due in days' and 'notes' are both optional
    'task name' must be in single or double quotes if more than one word
    example: # add 'Do Laundry' 1 [Probably 3 loads]
Delete a task:
    # delete 'task id'
    'task id' is the number(s) and/or range of numbers (i.e. 3-8)
    example: # delete 1 4 6-9 12";