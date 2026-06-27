use std::fs::{File, read_to_string};
use std::io;
use std::io::Write;

use crate::editor::terminal::TerminalResult;
use crate::editor::view::Location;

use super::line::Line;

#[derive(Default)]
pub struct Buffer {
    pub lines: Vec<Line>,
    file_name: Option<String>,
}

impl Buffer {
    pub fn load(file_path: &str) -> io::Result<Self> {
        let contents = read_to_string(file_path)?;
        let mut lines = Vec::new();
        for value in contents.lines() {
            lines.push(Line::from(value));
        }
        return Ok(Self {
            lines,
            file_name: Some(file_path.to_string()),
        });
    }

    pub fn save(&self) -> TerminalResult {
        if let Some(fp) = &self.file_name {
            let mut file = File::create(fp)?;
            for line in &self.lines {
                writeln!(file, "{line}")?;
            }
        }
        return Ok(());
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

    pub fn delete(&mut self, at: Location) {
        if let Some(line) = self.lines.get(at.line_idx) {
            if at.grapheme_idx >= line.grapheme_count()
                && at.line_idx.saturating_add(1) < self.height()
            {
                let next_line = self.lines.remove(at.line_idx.saturating_add(1));

                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_idx].append_line(&next_line);
            } else if at.grapheme_idx < line.grapheme_count() {
                #[allow(clippy::indexing_slicing)]
                self.lines[at.line_idx].delete_char(at.grapheme_idx);
            }
        }
    }

    pub fn insert_newline(&mut self, caret_pos: Location) {
        if let Some(line) = self.lines.get_mut(caret_pos.line_idx) {
            let new_line = line.split(caret_pos.grapheme_idx);
            self.lines
                .insert(caret_pos.line_idx.saturating_add(1), new_line);
        } else {
            self.lines.push(Line::default());
        }
    }
}
