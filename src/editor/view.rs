use std::cmp::min;

use crate::editor::{
    DocumentStatus, NAME, VERSION, command::Edit, uicomponent::UiComponent, view::line::Line,
};

use super::{
    command::Move,
    terminal::{IoResult, Position, Size, Terminal},
};
mod buffer;
use buffer::Buffer;
mod line;

#[derive(Copy, Clone, Default)]
pub struct Location {
    pub grapheme_idx: usize,
    pub line_idx: usize,
}

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    caret_location: Location,
    scroll_offset: Position,
}

impl From<Location> for Position {
    fn from(loc: Location) -> Self {
        return Self {
            col: loc.grapheme_idx,
            row: loc.line_idx,
        };
    }
}

impl View {
    pub fn get_status(&self) -> DocumentStatus {
        return DocumentStatus {
            file_name: format!("{}", self.buffer.file_info),
            line_count: self.buffer.height(),
            current_line: self.caret_location.line_idx,
            is_modified: self.buffer.unsaved,
        };
    }

    pub fn handle_edit_command(&mut self, edit: Edit) {
        match edit {
            Edit::Insert(ch) => self.insert_char(ch),
            Edit::NewLine => self.insert_newline(),
            Edit::Delete => self.delete(),
            Edit::Backspace => self.backspace(),
        }
    }

    pub fn has_unsaved_changed(&self) -> bool {
        return self.get_status().is_modified;
    }

    pub fn get_file_name(&self) -> String {
        return self
            .buffer
            .file_info
            .path
            .as_deref()
            .and_then(|p| return p.to_str())
            .unwrap_or("[No Name]")
            .to_string();
    }

    pub fn save(&mut self) -> IoResult {
        return self.buffer.save();
    }

    pub fn load(&mut self, file_name: &str) -> IoResult {
        let buffer = Buffer::load(file_name)?;
        self.buffer = buffer;
        self.mark_redraw(true);

        return Ok(());
    }

    fn insert_char(&mut self, c: char) {
        let line_idx = self.caret_location.line_idx;
        let old_len = self
            .buffer
            .lines
            .get(line_idx)
            .map_or(0, Line::grapheme_count);

        self.buffer.insert_char(c, self.caret_location);

        let new_len = self
            .buffer
            .lines
            .get(line_idx)
            .map_or(0, Line::grapheme_count);

        if new_len > old_len {
            self.handle_move_command(Move::Right);
        }

        self.mark_redraw(true);
    }

    fn delete(&mut self) {
        self.buffer.delete(self.caret_location);
        self.mark_redraw(true);
    }

    fn backspace(&mut self) {
        if self
            .caret_location
            .line_idx
            .saturating_add(self.caret_location.grapheme_idx)
            == 0
        {
            return;
        }
        self.handle_move_command(Move::Left);
        self.delete();
    }

    fn insert_newline(&mut self) {
        self.buffer.insert_newline(self.caret_location);
        self.handle_move_command(Move::Right);
        self.mark_redraw(true);
    }

    // rendering

    fn render_line(at: usize, line_text: &str) -> IoResult {
        return Terminal::print_row(at, line_text);
    }

    // scrolling

    pub fn scroll_vertically(&mut self, to: usize) {
        let Size { height, .. } = self.size;
        let offset_changed = if to < self.scroll_offset.row {
            self.scroll_offset.row = to;
            true
        } else if to >= self.scroll_offset.row.saturating_add(height) {
            self.scroll_offset.row = to.saturating_sub(height).saturating_add(1);
            true
        } else {
            false
        };
        self.mark_redraw(self.needs_redraw() || offset_changed);
    }

    pub fn scroll_horizontally(&mut self, to: usize) {
        let Size { width, .. } = self.size;
        let offset_changed = if to < self.scroll_offset.col {
            self.scroll_offset.col = to;
            true
        } else if to >= self.scroll_offset.col.saturating_add(width) {
            self.scroll_offset.col = to.saturating_sub(width).saturating_add(1);
            true
        } else {
            false
        };
        self.mark_redraw(self.needs_redraw() || offset_changed);
    }

    fn scroll(&mut self) {
        let Position { col, row } = self.text_location_as_position();
        self.scroll_vertically(row);
        self.scroll_horizontally(col);
    }

    pub fn caret_pos(&self) -> Position {
        return self
            .text_location_as_position()
            .saturating_sub(self.scroll_offset);
    }

    fn text_location_as_position(&self) -> Position {
        let row = self.caret_location.line_idx;
        let col = self.buffer.lines.get(row).map_or(0, |line| {
            return line.width_until(self.caret_location.grapheme_idx);
        });

        return Position { col, row };
    }

    pub fn handle_move_command(&mut self, direction: Move) {
        let Size { height, .. } = self.size;

        match direction {
            Move::PageUp => self.move_up(height.saturating_sub(1)),
            Move::PageDown => self.move_down(height.saturating_sub(1)),
            Move::StartOfLine => self.move_to_line_start(),
            Move::EndOfLine => self.move_to_line_end(),
            Move::Up => self.move_up(1),
            Move::Left => self.move_left(),
            Move::Right => self.move_right(),
            Move::Down => self.move_down(1),
        }
        self.scroll();
    }

    fn move_up(&mut self, step: usize) {
        self.caret_location.line_idx = self.caret_location.line_idx.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, step: usize) {
        self.caret_location.line_idx = self.caret_location.line_idx.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snap_to_valid_line();
    }

    fn move_left(&mut self) {
        #![allow(clippy::arithmetic_side_effects)]
        if self.caret_location.grapheme_idx > 0 {
            self.caret_location.grapheme_idx -= 1;
        } else if self.caret_location.line_idx > 0 {
            self.move_up(1);
            self.move_to_line_end();
        }
    }

    fn move_right(&mut self) {
        #![allow(clippy::arithmetic_side_effects)]
        let line_width = self
            .buffer
            .lines
            .get(self.caret_location.line_idx)
            .map_or(0, Line::grapheme_count);

        if self.caret_location.grapheme_idx < line_width {
            self.caret_location.grapheme_idx += 1;
        } else {
            self.move_to_line_start();
            self.move_down(1);
        }
    }

    fn move_to_line_start(&mut self) {
        self.caret_location.grapheme_idx = 0;
    }

    fn move_to_line_end(&mut self) {
        self.caret_location.grapheme_idx = self
            .buffer
            .lines
            .get(self.caret_location.line_idx)
            .map_or(0, Line::grapheme_count);
    }

    fn snap_to_valid_grapheme(&mut self) {
        self.caret_location.grapheme_idx = self
            .buffer
            .lines
            .get(self.caret_location.line_idx)
            .map_or(0, |line: &line::Line| {
                return min(line.grapheme_count(), self.caret_location.grapheme_idx);
            });
    }

    fn snap_to_valid_line(&mut self) {
        self.caret_location.line_idx = min(self.caret_location.line_idx, self.buffer.height());
    }

    fn build_welcome_message(width: usize) -> String {
        #![allow(clippy::integer_division)]
        if width == 0 {
            return String::new();
        }
        let welcome_message = format!("{NAME} editor -- version {VERSION}");
        let len = welcome_message.len();
        let remaining_width = width.saturating_sub(1);
        if remaining_width < len {
            return "~".to_string();
        }
        return format!("{:<1}{:^remaining_width$}", "~", welcome_message);
    }
}

impl Default for View {
    fn default() -> Self {
        return Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
            caret_location: Location::default(),
            scroll_offset: Position::default(),
        };
    }
}

impl UiComponent for View {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        return self.needs_redraw;
    }

    fn set_size(&mut self, size: Size) {
        let _ = Terminal::clear_screen();
        self.size = size;
        self.scroll();
    }

    fn draw(&mut self, origin_y: usize) -> IoResult {
        #![allow(clippy::integer_division)]
        let Size { height, width } = self.size;
        let end_y = origin_y.saturating_add(height);

        let top_third = height / 3;
        let scoll_top = self.scroll_offset.row;
        for current_row in origin_y..end_y {
            let line_idx = current_row
                .saturating_sub(origin_y)
                .saturating_add(scoll_top);

            if let Some(line) = self.buffer.lines.get(line_idx) {
                let left = self.scroll_offset.col;
                let right = self.scroll_offset.col.saturating_add(width);
                Self::render_line(current_row, &line.get_visible_graphemes(left..right))?;
            } else if current_row == top_third && self.buffer.is_empty() {
                Self::render_line(current_row, &Self::build_welcome_message(width))?;
            } else {
                Self::render_line(current_row, "~")?;
            }
        }

        return Ok(());
    }
}
