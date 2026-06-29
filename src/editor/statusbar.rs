use crossterm::style::Attribute;

use crate::editor::{DocumentStatus, terminal::Terminal, uicomponent::UiComponent};

use super::terminal::{IoResult, Size};

#[derive(Debug, Default)]
pub struct StatusBar {
    status: DocumentStatus,
    needs_redraw: bool,
    size: Size,
}

impl UiComponent for StatusBar {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        return self.needs_redraw;
    }

    fn set_size(&mut self, size: Size) {
        self.size = size;
    }

    fn draw(&mut self, origin_y: usize) -> IoResult {
        #![allow(clippy::integer_division)]
        let size = Terminal::size()?;

        let line_count = self.status.line_count_to_string();
        let modified = self.status.modified_indicator_to_string();
        let beginning = format!("{} - {} {}", self.status.file_name, line_count, modified);
        let position_indicator = self.status.position_indicator_to_string();

        let pos_len = position_indicator.len();
        let max_beginning = size.width.saturating_sub(pos_len);
        let beginning = if beginning.len() > max_beginning {
            beginning[..max_beginning].to_string()
        } else {
            beginning
        };

        let pad_len = size.width.saturating_sub(beginning.len());
        let status = format!("{beginning}{position_indicator:>pad_len$}");

        let status = if status.len() > size.width {
            " ".repeat(size.width)
        } else {
            status
        };

        Terminal::print_row_with_attribute(origin_y, &status, Attribute::Reverse)?;
        self.needs_redraw = false;
        return Ok(());
    }
}

impl StatusBar {
    pub fn update_status(&mut self, supplied_status: DocumentStatus) {
        if self.status == supplied_status {
            return;
        }
        self.status = supplied_status;
        self.mark_redraw(true);
    }
}
