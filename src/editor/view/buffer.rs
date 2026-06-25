use std::fs::read_to_string;
use std::io;

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
}
