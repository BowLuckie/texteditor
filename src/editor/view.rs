use crate::{
    TerminalResult,
    editor::{
        terminal::{Position, Size, Terminal as Term},
        view::buffer::Buffer,
    },
};

const VER: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

mod buffer;
#[derive(Default, Clone, Debug)]
pub struct View {
    buffer: Buffer,
}

impl View {
    pub fn render(&self) -> TerminalResult {
        #![allow(clippy::integer_division)]

        Term::move_cursor(Position { x: 0, y: 0 })?;

        let Size { height, .. } = Term::size()?;

        for current_row in 0..height {
            Term::clear_line()?;

            if let Some(line) = self.buffer.lines.get(current_row) {
                Term::print(line)?;
                Term::print("\r\n")?;
            } else {
                if current_row == height / 3 {
                    Self::draw_welcome_message()?;
                } else {
                    Self::draw_empty_row()?;
                }

                if current_row.saturating_add(1) < height {
                    Term::print("\r\n")?;
                }
            }
        }

        return Ok(());
    }

    fn draw_welcome_message() -> TerminalResult {
        #![allow(clippy::integer_division)]
        let mut welcome_message = format!("{NAME} editor -- version {VER}");
        let width = Term::size()?.width;
        let len = welcome_message.len();
        let padding = (width.saturating_sub(len)) / 2;
        let spaces = " ".repeat(padding.saturating_sub(1));
        welcome_message = format!("~{spaces}{welcome_message}");
        welcome_message.truncate(width);
        Term::print(&welcome_message)?;
        return Ok(());
    }

    fn draw_empty_row() -> TerminalResult {
        Term::print("~")?;
        return Ok(());
    }
}
