#![allow(unused)]

pub struct Command {
	add_or_delete: bool,
	description: String,
	due_in_days: Option<u8>,
	notes: Option<String>,
}

impl Command {
	pub const ADD: bool = true;
	pub const DELETE: bool = false;

	// noinspection Duplicates
	pub fn parse<T: ToString>(command: T) -> Result<Self, Box<dyn std::error::Error>> {
		let command = command.to_string();

		let (a_d, command) = command.split_once(" ").unwrap();
		let add_or_delete = match a_d {
			"a" | "add" => true,
			"d" | "delete" => false,
			_ => return Err("Could not parse command argument".into()),
		};

		let (command, notes) = extract_between(command, '[', ']');

		let notes = if !notes.is_empty() {
			Some(notes.to_string())
		} else {
			None
		};

		let (mut command, mut desc) = extract_between(command, '\'', '\'');
		if (desc.is_empty()) {
			(command, desc) = extract_between(command, '"', '"');
		}

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

		Ok(Self {
			add_or_delete,
			description,
			due_in_days,
			notes,
		})
	}
}

/// Extracts the text placed between the given delimiters, and removes excess white space left behind.
///
/// Example:
/// ```
/// let (remainder, extracted) = extract_between("some [other] text", '[', ']');
/// assert_eq!(String::from("some text"), remainder);
/// assert_eq!(String::from("other"), extracted);
/// ```
pub(crate) fn extract_between<T: ToString>(
	cmd: T,
	delim_a: char,
	delim_b: char,
) -> (String, String) {
	let cmd = cmd.to_string();
	let left = cmd.find(delim_a);
	let right = cmd.rfind(delim_b);

	if left.is_none() || right.is_none() {
		return (cmd, String::new());
	}

	if let Some(left) = left
		&& let Some(right) = right
		&& left < right
	{
		let extract = cmd[left + 1..right].to_string();

		// return the separated values
		(
			cmd.replace(&extract, "")
				.replace(delim_a, "")
				.replace(delim_b, "")
				.trim()
				.split_whitespace()
				.collect::<Vec<&str>>()
				.join(" "),
			extract,
		)
	} else {
		(cmd, String::new())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_extract_between() {
		let (c1, d1) = extract_between("this is a [1,2,3] test".to_string(), '[', ']');

		assert_eq!(c1, "this is a test");
		assert_eq!(d1, "1,2,3");

		let (c2, d2) = extract_between("'this 2' is a test".to_string(), '\'', '\'');
		assert_eq!(c2, "is a test");
		assert_eq!(d2, "this 2");

		let (c3, d3) = extract_between("'test 3' 3", '\'', '\'');

		assert_eq!(c3, "3");
		assert_eq!(d3, "test 3");

		// should fail and return original string and an empty string
		let (c4, d4) = extract_between("test 4", '\'', '\'');
		assert_eq!(c4, "test 4");
		assert_eq!(d4, "");

		// should fail and return original string and an empty string
		let (c5, d5) = extract_between("test [5", '[', ']');
		assert_eq!(c5, "test [5");
		assert_eq!(d5, "");

		let (c6, d6) = extract_between("\"test 6\" is a test", '\'', '\'');
		let (c6, d6) = extract_between(c6, '\"', '\"');
		assert_eq!(d6, "test 6");
		assert_eq!(c6, "is a test");
	}

	#[test]
	fn test_parse_command() {
		let c1 = Command::parse("a test").unwrap();
		assert_eq!(c1.add_or_delete, Command::ADD);
		assert_eq!(c1.description, "test");
		assert_eq!(c1.due_in_days, None);
		assert_eq!(c1.notes, None);

		let c2 = Command::parse("a 'test 2'").unwrap();
		assert_eq!(c2.add_or_delete, Command::ADD);
		assert_eq!(c2.description, "test 2");
		assert_eq!(c2.due_in_days, None);
		assert_eq!(c2.notes, None);

		let c3 = Command::parse("a test 3").unwrap();
		assert_eq!(c3.add_or_delete, Command::ADD);
		assert_eq!(c3.description, "test");
		assert_eq!(c3.due_in_days, Some(3));
		assert_eq!(c3.notes, None);

		let c4 = Command::parse("a 'test 4' 5 [notes]").unwrap();
		assert_eq!(c4.add_or_delete, Command::ADD);
		assert_eq!(c4.description, "test 4");
		assert_eq!(c4.due_in_days, Some(5));
		assert_eq!(c4.notes, Some(String::from("notes")));
	}
}
