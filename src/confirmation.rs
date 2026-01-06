/// Confirmation Dialog data struct. Persists the data needed to delete task(s) across UI update cycles.
pub struct ConfirmationDialog {
	tasks_to_delete: String,
	show: bool,
	confirm_cmd: String,
}

impl ConfirmationDialog {
	pub fn new() -> ConfirmationDialog {
		ConfirmationDialog {
			tasks_to_delete: String::new(),
			show: false,
			confirm_cmd: String::new(),
		}
	}

	pub fn title(&self) -> &str {
		"Confirm Delete Tasks"
	}

	pub fn text(&self) -> String {
		let binding = self.tasks_to_delete();
		let mut tasks = binding.split(' ').collect::<Vec<&str>>();

		let ttd = if tasks.len() > 1 {
			let last_task = tasks.pop().unwrap();
			let fmt_task = tasks.join(", ");

			format!("Delete Tasks {} and {}?\n(y/n)", fmt_task, last_task)
		} else {
			format!("Delete Task {}?\n(y/n)", tasks.first().unwrap())
		};

		let r = match self.tasks_to_delete.as_str() {
			"all" => "Delete All Tasks?\n(y/n)",
			_ => ttd.as_str(),
		};

		format!("{}\n{}", r, self.confirm_cmd)
	}
	pub fn is_shown(&self) -> bool {
		self.show
	}
	pub fn show(&mut self) {
		self.show = true
	}
	pub fn hide(&mut self) {
		self.show = false
	}

	pub fn tasks_to_delete(&self) -> String {
		self.tasks_to_delete.clone()
	}

	pub fn set_tasks_to_delete(&mut self, tasks_to_delete: String) {
		// process range input
		if tasks_to_delete.contains("-") {
			let mut tasks: Vec<String> = Vec::new();

			let clean_tasks = tasks_to_delete
				.replace(" - ", "-")
				.replace(" -", "-")
				.replace("- ", "-");

			for token in clean_tasks.split_whitespace() {
				if token.contains("-") {
					let (start_token, end_token) = token.split_once("-").unwrap();

					let start = start_token.trim().parse::<u32>().unwrap();
					let end = end_token.trim().parse::<u32>().unwrap();

					(start..=end).into_iter().for_each(|id| {
						tasks.push(id.to_string());
					})
				} else {
					tasks.push(token.to_string());
				}
			}

			/*let (start_token, end_token) = tasks_to_delete.split_once('-').unwrap();

			let start = start_token.trim().parse::<i32>().unwrap();
			let end = end_token.trim().parse::<i32>().unwrap();

			self.tasks_to_delete = (start..=end).into_iter().fold(String::new(), |acc, x| {
				format!("{} {}", acc, x)
			}).trim().to_string();*/

			self.tasks_to_delete = tasks
				.iter()
				.fold(String::new(), |acc, task| format!("{} {}", acc, task))
				.trim()
				.to_string()
		} else {
			self.tasks_to_delete = tasks_to_delete;
		}
	}

	pub fn reset(&mut self) {
		self.hide();
		self.set_tasks_to_delete(String::new());
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_tasks_to_delete() {
		let mut cd = ConfirmationDialog::new();

		cd.set_tasks_to_delete(String::from("7-10"));
		assert_eq!(cd.tasks_to_delete(), String::from("7 8 9 10"));

		cd.set_tasks_to_delete(String::from("1 2 3"));
		assert_eq!(cd.tasks_to_delete(), String::from("1 2 3"));

		cd.set_tasks_to_delete(String::from("1"));
		assert_eq!(cd.tasks_to_delete(), String::from("1"));

		cd.set_tasks_to_delete(String::from("2 4-6"));
		assert_eq!(cd.tasks_to_delete(), String::from("2 4 5 6"));

		cd.set_tasks_to_delete(String::from("2 4- 6 9"));
		assert_eq!(cd.tasks_to_delete(), String::from("2 4 5 6 9"));

		cd.set_tasks_to_delete(String::from("2 4 - 6 9"));
		assert_eq!(cd.tasks_to_delete(), String::from("2 4 5 6 9"));
	}
}
