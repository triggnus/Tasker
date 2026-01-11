#![allow(dead_code)]

use chrono::{Days, Local, NaiveDateTime};
use rusqlite::{Connection, params};
use std::fmt::Display;

pub struct Task {
	id: u32,
	description: String,
	notes: String,
	due_date: Option<NaiveDateTime>,
}

impl Task {
	pub fn new(
		id: u32,
		description: String,
		notes: String,
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

		let Some(due) = self.due_date else {
			return None;
		};

		Some(due.date().to_epoch_days() - now.to_epoch_days())
	}

	pub(crate) fn db_insert_task(
		conn: &Connection,
		description: String,
		notes: String,
		due_in_days: Option<u8>,
	) -> Result<Task, Box<dyn std::error::Error>> {
		let due_date = if let Some(due_days) = due_in_days {
			Local::now()
				.naive_local()
				.checked_add_days(Days::new(due_days as u64))
		} else {
			None
		};

		conn.execute(
			"INSERT INTO tasks (description, notes, due_date) VALUES (?1, ?2, ?3)",
			params![description, notes, due_date],
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

	// extract text between [ and ]
	fn extract_between(cmd: String, delim_a: char, delim_b: char) -> (String, String) {
		let left = cmd.find(delim_a);
		let right = cmd.rfind(delim_b);

		if left.is_none() || right.is_none() {
			return (cmd, String::new());
		}

		let notes = cmd[left.unwrap() + 1..right.unwrap()].to_string();
		(
			cmd.replace(&notes, "")
				.replace(delim_a, "")
				.replace(delim_b, "")
				.trim()
				.split_whitespace()
				.collect::<Vec<&str>>()
				.join(" "),
			notes,
		)
	}

	pub(crate) fn insert_task(
		conn: &Connection,
		command: String,
	) -> Result<Task, Box<dyn std::error::Error>> {
		let (command, notes) = Task::extract_between(command, '[', ']');
		let (command, desc) = Task::extract_between(command, '\'', '\'');

		let parts = command.rsplitn(2, ' ').collect::<Vec<&str>>();

		let description: String = if desc.is_empty() {
			if parts.len() == 1 {
				parts[0].to_string()
			} else {
				parts[1].to_string()
			}
		} else {
			desc
		};

		let due_in_days = parts[0].parse::<u8>().ok();

		Task::db_insert_task(conn, description, notes, due_in_days)
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
					Task::db_remove_task(&conn, task)?;
				}
				tasks.clear();
				break;
			} else if let Ok(num) = sub_command.parse::<u32>() {
				// delete task by number
				if let Some(position) = tasks.iter().position(|p| p.id() == num) {
					Task::db_remove_task(&conn, &tasks.remove(position))?
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

		let notes = if !self.notes.is_empty() {
			format!("Notes: {}", self.notes)
		} else {
			String::new()
		};

		let tail = match (due.is_empty(), notes.is_empty()) {
			(false, false) => format!(" {} {}", due, notes),
			(false, true) => format!(" {}", due),
			(true, false) => format!(" {}", notes),
			(true, true) => String::new()
		};

		write!(fmt, "[{}] '{}'{}", self.id, self.description, tail)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_extract_notes() {
		let (c, notes) = Task::extract_between("this is a [1,2,3] test".to_string(), '[',']');

		assert_eq!(c, "this is a test");
		assert_eq!(notes, "1,2,3");

		let (cmd, des) = Task::extract_between("'this 2' is a test".to_string(), '\'','\'');
		assert_eq!(des, "this 2");
		assert_eq!(cmd, "is a test");

		let (cmd, d2) = Task::extract_between("'test 3' 3".to_string(), '\'', '\'');

		println!("{cmd}");
		println!("{d2}");
	}
}
