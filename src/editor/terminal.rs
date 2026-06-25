use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::style::Print;
use crossterm::terminal::{
    Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode,
    enable_raw_mode, size,
};
use crossterm::{Command, queue};
use std::io::{Error, Write, stdout};

#[derive(Default, Copy, Clone)]
pub struct Size {
    pub height: usize,
    pub width: usize,
}

#[derive(Copy, Clone, Default)]
pub struct Position {
    pub col: usize,
    pub row: usize,
}

impl Position {
    pub const fn saturating_sub(self, other: Self) -> Self {
        return Self {
            row: self.row.saturating_sub(other.row),
            col: self.col.saturating_sub(other.col),
        };
    }
}

/// Represents the Terminal.
/// should you attempt to set the caret out of these bounds, it will be truncated.
pub struct Terminal;

pub type TerminalResult = std::io::Result<()>;

impl Terminal {
    pub fn terminate() -> TerminalResult {
        Self::leave_alternate_screen()?;
        Self::show_caret()?;
        Self::flush()?;
        disable_raw_mode()?;
        return Ok(());
    }
    pub fn initialize() -> TerminalResult {
        enable_raw_mode()?;
        Self::enter_alternate_screen()?;
        Self::clear_screen()?;
        Self::flush()?;
        return Ok(());
    }
    pub fn clear_screen() -> TerminalResult {
        Self::queue_command(Clear(ClearType::All))?;
        return Ok(());
    }
    pub fn clear_line() -> TerminalResult {
        Self::queue_command(Clear(ClearType::CurrentLine))?;
        return Ok(());
    }

    /// Moves the caret to the given Position.
    /// # Arguments
    /// * `Position` - the  `Position`to move the caret to. Will be truncated to `u16::MAX` if bigger.
    pub fn move_caret_to(position: Position) -> TerminalResult {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        Self::queue_command(MoveTo(position.col as u16, position.row as u16))?;
        return Ok(());
    }

    pub fn enter_alternate_screen() -> TerminalResult {
        Self::queue_command(EnterAlternateScreen)?;
        return Ok(());
    }

    pub fn leave_alternate_screen() -> TerminalResult {
        Self::queue_command(LeaveAlternateScreen)?;
        return Ok(());
    }

    pub fn hide_caret() -> TerminalResult {
        Self::queue_command(Hide)?;
        return Ok(());
    }

    pub fn show_caret() -> TerminalResult {
        Self::queue_command(Show)?;
        return Ok(());
    }

    pub fn print(string: &str) -> TerminalResult {
        Self::queue_command(Print(string))?;
        return Ok(());
    }

    pub fn print_row(row: usize, line_text: &str) -> TerminalResult {
        Self::move_caret_to(Position { row, col: 0 })?;
        Self::clear_line()?;
        Self::print(line_text)?;
        return Ok(());
    }

    /// Returns the current size of this Terminal.
    pub fn size() -> Result<Size, Error> {
        #![allow(clippy::as_conversions)]
        let (width_u16, height_u16) = size()?;
        let height = height_u16 as usize;
        let width = width_u16 as usize;
        return Ok(Size { height, width });
    }

    pub fn flush() -> TerminalResult {
        stdout().flush()?;
        return Ok(());
    }

    fn queue_command<T: Command>(command: T) -> TerminalResult {
        queue!(stdout(), command)?;
        return Ok(());
    }
}
