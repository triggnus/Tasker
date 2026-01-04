use crate::confirmation::ConfirmationDialog;
use crate::popup::{centered_rect, Popup};
use crate::task::Task;
use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
	execute,
	terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::style::{Color, Style};
use ratatui::{
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout},
	widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
	Terminal,
};
use rusqlite::Connection;
use std::io;
use ratatui::layout::{Offset, Position};

mod confirmation;
mod popup;
mod task;

//noinspection DuplicatedCode
fn main() -> Result<(), Box<dyn std::error::Error>> {
	let conn = Connection::open("db.sqlite").expect("Could not open db.sqlite");

	let mut stmt = conn
		.prepare("SELECT id, description, due_date FROM tasks ORDER BY id ASC")
		.expect("Could not prepare query");

	let mut tasks = stmt
		.query_map([], |row| {
			let task = Task::new(
				row.get("id").expect("Could not get id"),
				row.get("description").expect("Could not get description"),
				row.get("due_date").expect("Could not get due_date"),
			);

			Ok(task)
		})
		.expect("Could not load task(s)")
		.map(|task| task.unwrap())
		.collect::<Vec<Task>>();

	conn.execute(
		"CREATE TABLE IF NOT EXISTS tasks (
			id INTEGER PRIMARY KEY,
			description TEXT NOT NULL,
			due_date DATE NULL
		)",
		(),
	)
	.expect("Could not create table");

	// --- TERMINAL SETUP ---
	enable_raw_mode()?;
	let mut stdout = io::stdout();
	execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
	let backend = CrosstermBackend::new(stdout);
	let mut terminal = Terminal::new(backend)?;

	// --- APPLICATION STATE ---
	let mut input = String::new();
	let mut show_task_popup = false;
	let mut command = String::new();
	let mut task_popup_command = String::new();
	let mut confirm_dialog = ConfirmationDialog::new();

	loop {
		// --- UI DRAWING ---
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

			// 1. Render Input Box
			let input_widget = Paragraph::new(input.as_str()).block(
				Block::default()
					.borders(Borders::ALL)
					.title(" Enter Command (Enter) "),
			);
			f.render_widget(input_widget, chunks[0]);

			// 2. Render List
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

			if show_task_popup {
				let area = centered_rect(40, 20, size); // Dialog size: 60% width, 20% height
				f.render_widget(Clear, area); // This clears the background under the popup

				let title = format!(
					"{}{} Task",
					command.chars().next().unwrap().to_uppercase(),
					command.chars().skip(1).collect::<String>()
				);

				let command_color = if command == "add" {
					Color::Cyan
				} else {
					Color::Red
				};

				let popup = Popup::default()
					.style(Style::new().yellow())
					.title(title)
					//.centered(true)
					.content(task_popup_command.as_str())
					.title_style(Style::new().white().bold())
					.border_style(Style::new().fg(command_color));

				let mut cur_pos = area.as_position();
				cur_pos.x = cur_pos.x + task_popup_command.len() as u16 + 1;
				cur_pos.y += 1;

				f.set_cursor_position(cur_pos);

				f.render_widget(popup, area);
			}

			if confirm_dialog.is_shown() {
				let area = centered_rect(40, 20, size);
				f.render_widget(Clear, area);

				let popup = Popup::default()
					.style(Style::new().yellow())
					.title(confirm_dialog.title())
					.content(confirm_dialog.text())
					.centered(true)
					.title_style(Style::new().yellow().bold())
					.border_style(Style::new().red().bold());

				f.render_widget(popup, area);
			}

			if !show_task_popup && !confirm_dialog.is_shown() {
				f.set_cursor_position(Position::new(
					chunks[0].x + input.len() as u16 + 1,
					chunks[0].y + 1,
				));
			}
		})?;

		// --- INPUT HANDLING ---
		if let Event::Key(key) = event::read()? {
			match key.code {
				KeyCode::Esc => {
					if show_task_popup {
						show_task_popup = false;
					} else if confirm_dialog.is_shown() {
						confirm_dialog.hide();
					} else {
						break;
					}
				}
				KeyCode::Enter => {
					if !input.is_empty() {
						match input.to_lowercase().as_str() {
							"add" | "delete" => {
								show_task_popup = true;
								command = input.clone();
							}
							"a" => {
								show_task_popup = true;
								command = "add".to_string();
							}
							"d" => {
								show_task_popup = true;
								command = "delete".to_string();
							}
							"t" => {
								confirm_dialog.show();
							}
							"exit" | "quit" => break,
							_ => {}
						}

						if input.starts_with("d") {
							if let Some(sub_commands) = input.split_once(" ") {
								let tasks_to_delete = sub_commands.1.to_string();

								confirm_dialog.set_tasks_to_delete(tasks_to_delete.clone());
								confirm_dialog.show();
							}
						}

						if input.starts_with("a") {
							if let Some(sub_commands) = input.split_once(" ") {
								tasks.push(Task::insert_task(&conn, sub_commands.1.to_string())?);
							}
						}

						input.clear();
					}

					if show_task_popup && !task_popup_command.is_empty() {
						if command == "add" || command == "a" {
							let new_task = Task::insert_task(&conn, task_popup_command.clone())?;
							tasks.push(new_task);
						} else if command.starts_with("delete")
							|| command.chars().nth(1).unwrap() == 'd'
						{
							confirm_dialog.set_tasks_to_delete(task_popup_command.clone());
							confirm_dialog.show();
							//Task::remove_tasks(&conn, &mut tasks, task_popup_command.clone())?;
						}

						task_popup_command.clear();
						show_task_popup = false;
					}
				}
				KeyCode::Char(c) => {
					if show_task_popup {
						task_popup_command.push(c);
					} else if confirm_dialog.is_shown() {
						if c == 'y' {
							Task::remove_tasks(&conn, &mut tasks, confirm_dialog.tasks_to_delete()).expect("Could not remove tasks");
						}
						confirm_dialog.reset();
					} else {
						input.push(c);
					}
				}
				KeyCode::Backspace => {
					if show_task_popup {
						task_popup_command.pop();
					} else {
						input.pop();
					}
				}
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

#[cfg(test)]
mod tests {
	use super::*;

	//noinspection DuplicatedCode
	#[test]
	fn test_parse_task() -> Result<(), Box<dyn std::error::Error>> {
		let conn = Connection::open("db.sqlite").expect("Could not open db.sqlite");

		let mut stmt = conn
			.prepare("SELECT id, description, due_date FROM tasks ORDER BY id ASC")
			.expect("Could not prepare query");

		let mut tasks = stmt
			.query_map([], |row| {
				let task = Task::new(
					row.get("id").expect("Could not get id"),
					row.get("description").expect("Could not get description"),
					row.get("due_date").expect("Could not get due_date"),
				);

				Ok(task)
			})
			.expect("Could not load task(s)")
			.map(|task| task.unwrap())
			.collect::<Vec<Task>>();

		Task::insert_task(&conn, "add test1".to_string())?;
		Task::remove_tasks(&conn, &mut tasks, "delete all".to_string())?;

		Ok(())
	}
}
