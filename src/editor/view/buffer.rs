use std::fs::read_to_string;
use std::io;

use crate::editor::view::Location;

use super::line::Line;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
}

impl Buffer {
    pub fn load(file_name: &str) -> io::Result<Self> {
        let contents = read_to_string(file_name)?;
        let mut lines = Vec::new();
        for value in contents.lines() {
            lines.push(Line::from(value));
        }
        return Ok(Self { lines });
    }

    pub fn is_empty(&self) -> bool {
        return self.lines.is_empty();
    }

    pub fn height(&self) -> usize {
        return self.lines.len();
    }

    pub fn insert_char(&mut self, c: char, caret_location: Location) {
        let idx = caret_location.line_idx;
        if idx > self.lines.len() {
            return;
        }

        if idx == self.lines.len() || self.lines.is_empty() {
            self.lines.push(Line::from(""));
        }

        let line = self.lines.get_mut(idx).unwrap();
        line.insert_char(c, caret_location.grapheme_idx);
    }
}
