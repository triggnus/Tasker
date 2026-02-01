/// Types that implement CharToVec promise the ability to turn whatever data they hold into a [Vec\<char\>]
pub trait CharToVec {
	fn to_vec(&self) -> Vec<char>;
}

impl CharToVec for char {
	fn to_vec(&self) -> Vec<char> {
		vec![*self]
	}
}

impl<const N: usize> CharToVec for [char; N] {
	fn to_vec(&self) -> Vec<char> {
		let mut list = vec![];

		for e in self {
			list.push(e.to_owned());
		}

		list
	}
}

impl CharToVec for String {
	fn to_vec(&self) -> Vec<char> {
		self.chars().collect()
	}
}

impl CharToVec for &str {
	fn to_vec(&self) -> Vec<char> {
		self.chars().collect()
	}
}