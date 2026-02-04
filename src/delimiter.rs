#![allow(unused)]

use crate::char_to_vec::CharToVec;

pub trait Delimited {
	fn contains_delimiter(&self, delim: &Delimiter) -> bool;
	fn get_delimiter_positions(&self, delim: &Delimiter) -> Vec<usize>;
}

impl Delimited for String {
	fn contains_delimiter(&self, delim: &Delimiter) -> bool {
		for d in &delim.delimiters {
			if !self.contains(*d) {
				return false;
			}
		}

		true
	}
	fn get_delimiter_positions(&self, delim: &Delimiter) -> Vec<usize> {
		delim.find_all(self.to_owned())
	}
}

pub struct Delimiter {
	delimiters: Vec<char>,
}

impl Delimiter {
	pub fn new_single_delimiter(delimiter: char) -> Self {
		Self {
			delimiters: vec![delimiter],
		}
	}

	pub fn new_multi_delimiter(delimiters: impl CharToVec) -> Self {
		Self {
			delimiters: delimiters.to_vec(),
		}
	}

	/// Finds all occurrences of the delimiter(s) in a given string, or returns None if none are found.
	pub fn find_all(&self, text: String) -> Vec<usize> {
		let mut found_positions: Vec<usize> = Vec::new();

		for (idx, c) in text.chars().enumerate() {
			if self.delimiters.contains(&c) {
				found_positions.push(idx);
			}
		}

		found_positions
	}

	pub fn get_delimiters(&self) -> &Vec<char> {
		&self.delimiters
	}
}

#[cfg(test)]
mod tests {
	use rusqlite::fallible_iterator::FallibleIterator;
	use super::*;

	#[test]
	fn test_delimiter() {
		let d1 = Delimiter::new_single_delimiter('\'');
		assert_eq!(d1.get_delimiters(), &vec!['\'']);

		let t1 = d1.find_all("this is 'a test' of delimiters".to_string());
		assert!(!t1.is_empty());
		assert_eq!(t1, vec![8_usize, 15_usize]);

		let d2 = Delimiter::new_single_delimiter('"');
		assert_eq!(d2.get_delimiters(), &vec!['"']);

		let d3 = Delimiter::new_multi_delimiter(['[', ']']);
		assert_eq!(d3.get_delimiters(), &vec!('[', ']'));

		let t3 = d3.find_all("Testing a [multiple delimiter] search".to_string());
		assert!(!t3.is_empty());
		assert_eq!(t3, vec![10_usize, 29_usize]);

		let d4 = Delimiter::new_multi_delimiter(['!']);
		let t4 = d4.find_all("Testing a search where 'delimiter' wasn't found".to_string());
		assert!(t4.is_empty());

		let d5 = Delimiter::new_multi_delimiter(['[', ']']);
		let d5_2 = Delimiter::new_multi_delimiter(['[', ']', '!']);
		let s5 = String::from("This is a [multiple delimiter] search");
		assert!(s5.contains_delimiter(&d5));
		assert!(!s5.contains_delimiter(&d5_2));
		assert_eq!(s5.get_delimiter_positions(&d5), vec![10_usize, 29_usize]);
	}
}