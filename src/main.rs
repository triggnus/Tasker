use crate::popup::{Popup, centered_rect};
use crate::task::Task;
use clap::Parser;
use crossterm::{
	event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
	execute,
	terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::style::{Color, Style};
use ratatui::{
	Terminal,
	backend::CrosstermBackend,
	layout::{Constraint, Direction, Layout},
	widgets::{Block, Borders, Clear, List, ListItem, Paragraph},
};
use rusqlite::{Connection, params};
use std::io;

mod popup;
mod task;

/// Simple command line task manager.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None, version="0.1")] // Generates help and version info
struct Args {
	/// Name of the task
	#[arg(short, long)]
	task: Option<String>,

	/// Days until due
	#[arg(short, long)]
	due_in_days: Option<u8>,

	/// Print active tasks
	#[arg(short, long)]
	print: bool,

	/// Complete a given task
	#[arg(short, long)]
	complete: Option<u32>,

	/// Interactive Mode
	#[arg(short, long)]
	interactive: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
	let args = Args::parse();

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

	if let Some(t) = args.task {
		Task::db_insert_task(&conn, t, args.due_in_days).expect("Could not insert task");
	}

	if let Some(complete) = args.complete {
		conn.execute("DELETE FROM tasks WHERE id = ?1", params![complete])
			.expect("Could not delete task");
	}

	if args.print {
		for task in &tasks {
			println!("{}", task);
		}
	}

	if args.interactive {
		// --- TERMINAL SETUP ---
		enable_raw_mode()?;
		let mut stdout = io::stdout();
		execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
		let backend = CrosstermBackend::new(stdout);
		let mut terminal = Terminal::new(backend)?;

		// --- APPLICATION STATE ---
		let mut input = String::new();
		let mut command_input = String::new();
		let mut show_popup = false;
		let mut command = String::new();

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
				let list_items: Vec<ListItem> =
					tasks.iter().map(|i| ListItem::new(i.to_string())).collect();

				let list_widget = List::new(list_items).block(
					Block::default()
						.borders(Borders::ALL)
						.title(" Current Tasks (Esc to Quit) ")
						.style(Style::new().bg(Color::DarkGray).fg(Color::White)),
				);
				f.render_widget(list_widget, chunks[1]);

				if show_popup {
					let area = centered_rect(60, 20, size); // Dialog size: 60% width, 20% height
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
						.content(command_input.as_str())
						.title_style(Style::new().white().bold())
						.border_style(Style::new().fg(command_color));

					f.render_widget(popup, area);
				}
			})?;

			// --- INPUT HANDLING ---
			if let Event::Key(key) = event::read()? {
				match key.code {
					KeyCode::Esc => {
						if show_popup {
							show_popup = false;
						} else {
							break
						}
					},
					KeyCode::Enter => {
						if !input.is_empty() {
							match input.to_lowercase().as_str() {
								"add" | "delete" => {
									show_popup = true;
									command = input.clone();
								}
								"a" => {
									show_popup = true;
									command = "add".to_string();
								}
								"d" => {
									show_popup = true;
									command = "delete".to_string();
								}
								"exit" | "quit" => break,
								_ => {}
							}

							if input.starts_with("d") {
								if let Some(sub_commands) = input.split_once(" ") {
									Task::remove_tasks(
										&conn,
										&mut tasks,
										sub_commands.1.to_string(),
									)?;
								}
							}

							if input.starts_with("a") {
								if let Some(sub_commands) = input.split_once(" ") {
									tasks.push(Task::insert_task(
										&conn,
										sub_commands.1.to_string(),
									)?);
								}
							}

							input.clear();
						}

						if show_popup && !command_input.is_empty() {
							if command == "add" || command == "a" {
								let new_task = Task::insert_task(&conn, command_input.clone())?;
								tasks.push(new_task);
							} else if command.starts_with("delete")
								|| command.chars().nth(1).unwrap() == 'd'
							{
								Task::remove_tasks(&conn, &mut tasks, command_input.clone())?;
							}

							command_input.clear();
							show_popup = false;
						}
					}
					KeyCode::Char(c) => {
						if show_popup {
							command_input.push(c);
						} else {
							input.push(c);
						}
					}
					KeyCode::Backspace => {
						if show_popup {
							command_input.pop();
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
	}

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
