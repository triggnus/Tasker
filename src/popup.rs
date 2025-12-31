use derive_setters::Setters;

use ratatui::layout::{Constraint, Direction, Layout};
use ratatui::{
	buffer::Buffer,
	layout::Rect,
	style::Style,
	text::{Line, Text},
	widgets::{Block, Borders, Clear, Paragraph, Widget, Wrap},
};

#[derive(Debug, Default, Setters)]
pub struct Popup<'a> {
	#[setters(into)]
	title: Line<'a>,
	#[setters(into)]
	content: Text<'a>,
	border_style: Style,
	title_style: Style,
	style: Style,
}

impl Widget for Popup<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		// ensure that all cells under the popup are cleared to avoid leaking content
		Clear.render(area, buf);
		let block = Block::new()
			.title(self.title)
			.title_style(self.title_style)
			.borders(Borders::ALL)
			.border_style(self.border_style);
		Paragraph::new(self.content)
			.wrap(Wrap { trim: true })
			.style(self.style)
			.block(block)
			.render(area, buf);
	}
}

/// Helper function to create a centered rectangle for the dialog
//noinspection DuplicatedCode
pub fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
	let popup_layout = Layout::default()
		.direction(Direction::Vertical)
		.constraints([
			Constraint::Percentage((100 - percent_y) / 2),
			Constraint::Percentage(percent_y),
			Constraint::Percentage((100 - percent_y) / 2),
		])
		.split(r);

	Layout::default()
		.direction(Direction::Horizontal)
		.constraints([
			Constraint::Percentage((100 - percent_x) / 2),
			Constraint::Percentage(percent_x),
			Constraint::Percentage((100 - percent_x) / 2),
		])
		.split(popup_layout[1])[1]
}
