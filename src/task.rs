#![allow(dead_code)]

use crate::command::InsertCommand;
use chrono::{Days, Local, NaiveDateTime};
use rusqlite::{Connection, params};
use std::fmt::Display;

pub struct Task {
	id: u32,
	description: String,
	notes: Option<String>,
	due_date: Option<NaiveDateTime>,
}

impl Task {
	pub fn new(
		id: u32,
		description: String,
		notes: Option<String>,
		due_date: Option<NaiveDateTime>,
	) -> Self {
		Self {
			id,
			description,
			notes,
			due_date,
		}
	}

	pub(crate) fn id(&self) -> u32 {
		self.id
	}

	pub(crate) fn due_in_days(&self) -> Option<i32> {
		let now = Local::now().date_naive();

		let due = self.due_date?;

		Some(due.date().to_epoch_days() - now.to_epoch_days())
	}

	pub(crate) fn db_insert_task(
		conn: &Connection,
		insert_command: InsertCommand,
	) -> Result<Task, Box<dyn std::error::Error>> {
		let due_date = if let Some(due_days) = insert_command.due_in_days {
			Local::now()
				.naive_local()
				.checked_add_days(Days::new(due_days as u64))
		} else {
			None
		};

		conn.execute(
			"INSERT INTO tasks (description, notes, due_date) VALUES (?1, ?2, ?3)",
			params![insert_command.description, insert_command.notes, due_date],
		)?;

		let mut stmt = conn.prepare(
			"SELECT id, description, notes, due_date FROM tasks ORDER BY id DESC LIMIT 1",
		)?;

		let task = stmt.query_one([], |row| {
			Ok(Task::new(
				row.get("id")?,
				row.get("description")?,
				row.get("notes")?,
				row.get("due_date")?,
			))
		})?;

		Ok(task)
	}

	pub(crate) fn db_remove_task(
		conn: &Connection,
		task: &Task,
	) -> Result<(), Box<dyn std::error::Error>> {
		conn.execute("DELETE FROM tasks WHERE id = ?1", params![task.id])?;

		Ok(())
	}

	// noinspection Duplicates
	pub(crate) fn insert_task(
		conn: &Connection,
		command: String,
	) -> Result<Task, Box<dyn std::error::Error>> {
		let parsed_command = InsertCommand::parse(&command)?;
		//let notes = parsed_command.notes.unwrap_or(String::new());

		Task::db_insert_task(
			conn,
			parsed_command,
		)
	}

	pub(crate) fn remove_tasks(
		conn: &Connection,
		tasks: &mut Vec<Task>,
		command: String,
	) -> Result<(), Box<dyn std::error::Error>> {
		let sub_commands: Vec<&str> = command.split(' ').collect();

		for sub_command in sub_commands {
			if sub_command == "all" {
				// delete all tasks
				for task in tasks.iter() {
					Task::db_remove_task(conn, task)?;
				}
				tasks.clear();
				break;
			} else if let Ok(num) = sub_command.parse::<u32>() {
				// delete task by number
				if let Some(position) = tasks.iter().position(|p| p.id() == num) {
					Task::db_remove_task(conn, &tasks.remove(position))?
				}
			}
		}

		Ok(())
	}
}

impl Display for Task {
	fn fmt(&self, fmt: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
		let due = if let Some(due_in_days) = self.due_in_days() {
			if due_in_days > 0 {
				format!(
					"due in {} days ({})",
					due_in_days,
					// SAFETY: if due_in_days() is Some, then due_date must also be Some, so we can safely unwrap
					self.due_date.unwrap().format("%m-%d-%Y")
				)
			} else if due_in_days == 0 {
				String::from("Due Today")
			} else {
				format!("PAST DUE ({} days)", -due_in_days)
			}
		} else {
			String::new()
		};

		let notes = if let Some(notes) = &self.notes {
			format!("Notes: {}", notes)
		} else {
			String::new()
		};

		let tail = match (due.is_empty(), notes.is_empty()) {
			(false, false) => format!(" {} {}", due, notes),
			(false, true) => format!(" {}", due),
			(true, false) => format!(" {}", notes),
			(true, true) => String::new(),
		};

		write!(fmt, "[{}] '{}'{}", self.id, self.description, tail)
	}
}
