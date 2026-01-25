#![allow(unused)]

use crate::char_to_vec::CharToVec;

pub struct InsertCommand {
	pub description: String,
	pub due_in_days: Option<u8>,
	pub notes: Option<String>,
}

impl InsertCommand {
	/// Parses an input string into a corresponding set of commands.
	// noinspection Duplicates
	pub fn parse<T: ToString>(command: T) -> Result<Self, Box<dyn std::error::Error>> {
		let command = command.to_string();

		let mut command = command.to_string();

		let mut notes = String::new();

		//let (command, notes) = extract_between(command, '[', ']');
		let parsed_command = parse_by_delimiter(&mut command, ['[', ']']);
		if let Ok(parsed_command) = parsed_command {
			command = parsed_command.remaining;
			notes = parsed_command.extracted;
		}

		let notes = if !notes.is_empty() {
			Some(notes.to_string())
		} else {
			None
		};

		let mut desc = String::new();

		let parsed_command = parse_by_delimiter(&mut command, '\'');
		if let Ok(parsed_command) = parsed_command {
			command = parsed_command.remaining;
			desc = parsed_command.extracted;
		}

		if (desc.is_empty()) {
			//(command, desc) = extract_between(command, '"', '"');
			let parsed_command = parse_by_delimiter(&mut command, '"');
			if let Ok(parsed_command) = parsed_command {
				command = parsed_command.remaining;
				desc = parsed_command.extracted;
			}
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
			description,
			due_in_days,
			notes,
		})
	}
}

pub struct ParseResult {
	pub initial: String,
	pub extracted: String,
	pub remaining: String,
}

pub(crate) fn parse_by_delimiter(command: impl ToString, delimiter: impl CharToVec) -> Result<ParseResult, Box<dyn std::error::Error>> {
	let command = command.to_string();

	if !command.contains(delimiter.to_vec().as_slice()) {
		return Err("Delimiter Not Found!".into());
	}

	let d_vec = delimiter.to_vec();

	let delim_a = d_vec[0];
	let delim_b = if d_vec.len() == 2 { d_vec[1] } else { d_vec[0] };

	let left = command.find(delim_a);
	let right = command.rfind(delim_b);

	if left.is_none() || right.is_none() {
		return Err(format!("Could not find a matching delimiters in command ({})", command).into());
	}

	// SAFETY: we can safely unwrap left and right, because we test that neither was none in the previous step.
	let mut left = left.unwrap();
	let mut right = right.unwrap();

	// in theory, left should never occur after right, but if it does, just swap them.
	if left > right {
		std::mem::swap(&mut left, &mut right);
	}

	let extract = command[left + 1..right].to_string();

	Ok(ParseResult{
		initial: command.to_owned(),
		extracted: extract.to_owned(),
		remaining: command.replace(&extract, "")
		.replace(d_vec[0], "")
		.replace(delim_b, "")
		.trim()
		.split_whitespace()
		.collect::<Vec<&str>>()
		.join(" "),
	})
}


#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_parse_by_delimiter() {
		let a1 = parse_by_delimiter("this is a [1,2,3] test", ['[', ']']);
		assert!(a1.is_ok());
		let b1 = a1.unwrap();
		assert_eq!(b1.extracted, String::from("1,2,3"));
		assert_eq!(b1.remaining, String::from("this is a test"));

		let a2 = parse_by_delimiter("this test [fails", ['[', ']']);
		assert!(a2.is_err());

		let a3 = parse_by_delimiter("this is 'quoted' text", '\'');
		assert!(a3.is_ok());
		let b3 = a3.unwrap();
		assert_eq!(b3.extracted, String::from("quoted"));
		assert_eq!(b3.remaining, String::from("this is text"));

		let a4 = parse_by_delimiter("this is a [1,2,3] test", "[]");
		assert!(a4.is_ok());
		let b4 = a4.unwrap();
		assert_eq!(b4.extracted, String::from("1,2,3"));
		assert_eq!(b4.remaining, String::from("this is a test"));
	}

	#[test]
	fn test_parse_command() {
		let c1 = InsertCommand::parse("a test").unwrap();
		assert_eq!(c1.description, "test");
		assert_eq!(c1.due_in_days, None);
		assert_eq!(c1.notes, None);

		let c2 = InsertCommand::parse("a 'test 2'").unwrap();
		assert_eq!(c2.description, "test 2");
		assert_eq!(c2.due_in_days, None);
		assert_eq!(c2.notes, None);

		let c3 = InsertCommand::parse("a test 3").unwrap();
		assert_eq!(c3.description, "test");
		assert_eq!(c3.due_in_days, Some(3));
		assert_eq!(c3.notes, None);

		let c4 = InsertCommand::parse("a 'test 4' 5 [notes]").unwrap();
		assert_eq!(c4.description, "test 4");
		assert_eq!(c4.due_in_days, Some(5));
		assert_eq!(c4.notes, Some(String::from("notes")));
	}
}
