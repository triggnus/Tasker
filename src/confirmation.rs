
pub struct ConfirmationDialog {
	tasks_to_delete: String,
	show: bool,
	confirm_cmd: String,
}

impl ConfirmationDialog {
	pub fn new() -> ConfirmationDialog
	{
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
		self.tasks_to_delete = tasks_to_delete;
	}

	pub fn reset(&mut self) {
		self.hide();
		self.set_tasks_to_delete(String::new());
	}
}
