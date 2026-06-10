use crate::{IoErr, TerminalResult};
use std::{
    fmt::Display,
    io::{self, Write, stdout},
};

use crossterm::{
    Command,
    cursor::{Hide, MoveTo, Show},
    execute, queue,
    style::{Attribute::SlowBlink, Print},
    terminal::{
        Clear,
        ClearType::{self, CurrentLine},
        disable_raw_mode, enable_raw_mode,
    },
};

#[derive(Debug, Clone, Copy)]
pub struct Size {
    pub height: usize,
    pub width: usize,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Position {
    pub x: usize,
    pub y: usize,
}

/// represents the terminal.
/// undefined behaviour if `usize` is smaller than `u16`.
pub struct Terminal;

impl Terminal {
    pub fn initialize() -> TerminalResult {
        enable_raw_mode()?;
        Self::clear_screen()?;
        Self::move_cursor(Position { x: 0, y: 0 })?;
        Self::flush()?;
        return Ok(());
    }

    pub fn clear_screen() -> TerminalResult {
        queue_command(Clear(ClearType::All))
    }

    pub fn clear_line() -> TerminalResult {
        return queue_command(Clear(ClearType::CurrentLine));
    }

    pub fn terminate() -> TerminalResult {
        Self::flush()?;
        disable_raw_mode()?;
        return Ok(());
    }

    /// moves the cursor to a given position `position`.
    /// position will be truncated if `u16::MAX` is exceeded.
    pub fn move_cursor(position: Position) -> TerminalResult {
        #[allow(clippy::as_conversions, clippy::cast_possible_truncation)]
        return queue_command(MoveTo(position.x as u16, position.y as u16));
    }

    pub fn flush() -> TerminalResult {
        stdout().flush()?;
        return Ok(());
    }

    pub fn size() -> io::Result<Size> {
        #![allow(clippy::as_conversions)]
        let (width, height) = crossterm::terminal::size()?;
        Ok(Size {
            height: height as usize,
            width: width as usize,
        })
    }

    pub fn show_cursor() -> TerminalResult {
        return queue_command(Show);
    }

    pub fn hide_cursor() -> TerminalResult {
        return queue_command(Hide);
    }

    /// preforms an unflushed print on the current stdout session.
    pub fn print<T: Display>(string: T) -> TerminalResult {
        return queue_command(Print(string));
    }
}

fn queue_command<T: Command>(command: T) -> TerminalResult {
    queue!(stdout(), command)?;
    return Ok(());
}
