use std::cmp::min;

use crate::editor::view::line::Line;

use super::{
    editorcommand::{Direction, EditorCommand},
    terminal::{Position, Size, Terminal},
};
mod buffer;
use buffer::Buffer;
mod line;
const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Copy, Clone, Default)]
pub struct Location {
    pub grapheme_idx: usize,
    pub line_idx: usize,
}

pub struct View {
    buffer: Buffer,
    needs_redraw: bool,
    size: Size,
    text_location: Location,
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
    pub fn handle_command(&mut self, command: EditorCommand) {
        match command {
            EditorCommand::Resize(size) => self.resize(size),
            EditorCommand::Move(direction) => {
                self.move_text_location(&direction);
                self.scroll();
            }
            EditorCommand::Quit => {}
        }
    }

    pub fn load(&mut self, file_name: &str) {
        if let Ok(buffer) = Buffer::load(file_name) {
            self.buffer = buffer;
            self.needs_redraw = true;
        }
    }

    fn resize(&mut self, to: Size) {
        self.size = to;
        self.scroll();
        self.needs_redraw = true;
    }

    // rendering

    pub fn render(&mut self) {
        #![allow(clippy::integer_division)]
        if !self.needs_redraw {
            return;
        }

        let Size { height, width } = self.size;
        if height == 0 || width == 0 {
            return;
        }

        let vertical_center = height / 3;
        let top = self.scroll_offset.row;
        for current_row in 0..height {
            if let Some(line) = self.buffer.lines.get(current_row.saturating_add(top)) {
                let left = self.scroll_offset.col;
                let right = self.scroll_offset.col.saturating_add(width);
                Self::render_line(current_row, &line.get_visible_graphemes(left..right));
            } else if current_row == vertical_center && self.buffer.is_empty() {
                Self::render_line(current_row, &Self::build_welcome_message(width));
            } else {
                Self::render_line(current_row, "~");
            }
        }
        self.needs_redraw = false;
    }

    fn render_line(at: usize, line_text: &str) {
        let result = Terminal::print_row(at, line_text);
        debug_assert!(result.is_ok(), "Failed to render line");
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
        self.needs_redraw = self.needs_redraw || offset_changed;
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
        self.needs_redraw = self.needs_redraw || offset_changed;
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
        let row = self.text_location.line_idx;
        let col = self.buffer.lines.get(row).map_or(0, |line| {
            return line.width_until(self.text_location.grapheme_idx);
        });

        return Position { col, row };
    }

    fn move_text_location(&mut self, direction: &Direction) {
        let Size { height, .. } = self.size;

        match direction {
            Direction::PageUp => self.move_up(height.saturating_sub(1)),
            Direction::PageDown => self.move_down(height.saturating_sub(1)),
            Direction::Home => self.move_to_line_start(),
            Direction::End => self.move_to_line_end(),
            Direction::Up => self.move_up(1),
            Direction::Left => self.move_left(),
            Direction::Right => self.move_right(),
            Direction::Down => self.move_down(1),
        }
    }

    fn move_up(&mut self, step: usize) {
        self.text_location.line_idx = self.text_location.line_idx.saturating_sub(step);
        self.snap_to_valid_grapheme();
    }

    fn move_down(&mut self, step: usize) {
        self.text_location.line_idx = self.text_location.line_idx.saturating_add(step);
        self.snap_to_valid_grapheme();
        self.snape_to_valid_line();
    }

    fn move_left(&mut self) {
        #![allow(clippy::arithmetic_side_effects)]
        if self.text_location.grapheme_idx > 0 {
            self.text_location.grapheme_idx -= 1;
        } else if self.text_location.line_idx > 0 {
            self.move_up(1);
            self.move_to_line_end();
        }
    }

    fn move_right(&mut self) {
        #![allow(clippy::arithmetic_side_effects)]
        let line_width = self
            .buffer
            .lines
            .get(self.text_location.line_idx)
            .map_or(0, Line::grapheme_len);

        if self.text_location.grapheme_idx < line_width {
            self.text_location.grapheme_idx += 1;
        } else {
            self.move_to_line_start();
            self.move_down(1);
        }
    }

    fn move_to_line_start(&mut self) {
        self.text_location.grapheme_idx = 0;
    }

    fn move_to_line_end(&mut self) {
        self.text_location.grapheme_idx = self
            .buffer
            .lines
            .get(self.text_location.line_idx)
            .map_or(0, Line::grapheme_len);
    }

    fn snap_to_valid_grapheme(&mut self) {
        self.text_location.grapheme_idx = self
            .buffer
            .lines
            .get(self.text_location.line_idx)
            .map_or(0, |line: &line::Line| {
                return min(line.grapheme_len(), self.text_location.grapheme_idx);
            });
    }

    fn snape_to_valid_line(&mut self) {
        self.text_location.line_idx = min(self.text_location.line_idx, self.buffer.height());
    }

    fn build_welcome_message(width: usize) -> String {
        #![allow(clippy::integer_division)]
        if width == 0 {
            return " ".to_string();
        }
        let welcome_message = format!("{NAME} editor -- version {VERSION}");
        let len = welcome_message.len();
        if width <= len {
            return "~".to_string();
        }

        let padding = (width.saturating_sub(len).saturating_sub(1)) / 2;

        let mut full_message = format!("~{}{}", " ".repeat(padding), welcome_message);
        full_message.truncate(width);
        return full_message;
    }
}

impl Default for View {
    fn default() -> Self {
        return Self {
            buffer: Buffer::default(),
            needs_redraw: true,
            size: Terminal::size().unwrap_or_default(),
            text_location: Location::default(),
            scroll_offset: Position::default(),
        };
    }
}
