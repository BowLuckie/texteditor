use std::{
    default,
    io::{self, Read, Write, stdout},
    sync::Arc,
};

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{
        self,
        Event::{self, Key},
        KeyCode::{self, Char},
        KeyEvent,
        KeyEventKind::Press,
        KeyModifiers,
    },
    execute,
    terminal::{Clear, ClearType, disable_raw_mode, enable_raw_mode},
};
use event::KeyCode::{Down, Left, Right, Up};

use crate::{
    IoErr, TerminalResult,
    editor::terminal::{Position, Size},
};

mod terminal;
use terminal::Terminal as Term;

/// represents a complete editor.
#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    cursor_location: Position,
}

const VER: &str = env!("CARGO_PKG_VERSION");
const NAME: &str = env!("CARGO_PKG_NAME");

impl Editor {
    /// Creates a new [`Editor`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Runs this [`Editor`] by rendering it in the terminal.
    pub fn run(&mut self) {
        Term::initialize().unwrap();
        let result = self.repl();
        Term::terminate().unwrap();
        result.unwrap();
    }

    fn repl(&mut self) -> TerminalResult {
        loop {
            self.refresh_screen()?;
            if self.should_quit {
                break;
            }
            let event = event::read()?;
            self.evaluate_event(&event);
        }
        Ok(())
    }

    fn evaluate_event(&mut self, event: &Event) {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
            && let Char('q') = code
            && modifiers == &KeyModifiers::CONTROL
        {
            self.should_quit = true;
        }

        if let Key(KeyEvent {
            code,
            modifiers,
            kind: Press,
            ..
        }) = event
            && let Up | Down | Left | Right = *code
        {
            self.directional_move(*code);
        }
    }

    fn refresh_screen(&self) -> TerminalResult {
        Term::hide_cursor()?;
        if self.should_quit {
            Term::clear_screen()?;
            Term::print("Goodbye.\r\n")?;
        } else {
            Self::draw_rows()?;
            Term::move_cursor(self.cursor_location)?;
        }
        Term::show_cursor()?;
        Term::flush()?;
        Ok(())
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
        Term::print(welcome_message)?;
        Ok(())
    }

    fn draw_empty_row() -> TerminalResult {
        Term::print("~")?;
        Ok(())
    }

    fn draw_rows() -> TerminalResult {
        #![allow(clippy::integer_division)]
        Term::move_cursor(Position { x: 0, y: 0 });
        let Size { height, .. } = Term::size()?;
        for current_row in 0..height {
            Term::clear_line()?;
            if current_row == height / 3 {
                Self::draw_welcome_message()?;
            } else {
                Self::draw_empty_row()?;
            }
            if current_row.saturating_add(1) < height {
                Term::print("\r\n")?;
            }
        }
        return Ok(());
    }

    fn move_cursor_location(&mut self, position: Position) {
        // TODO:
        self.cursor_location = position;
    }

    fn directional_move(&mut self, direction: KeyCode) -> TerminalResult {
        let (x, y) = (self.cursor_location.x, self.cursor_location.y);
        let Size {
            width: size_x,
            height: size_y,
        } = Term::size()?;
        match direction {
            Up if y != 0 => self.move_cursor_location(Position { x, y: y - 1 }),
            Down if size_y - 1 != y => self.move_cursor_location(Position { x, y: y + 1 }),
            Left if x != 0 => self.move_cursor_location(Position { x: x - 1, y }),
            Right if x != size_x - 1 => self.move_cursor_location(Position { x: x + 1, y }),
            _ => return Ok(()),
        }

        return Ok(());
    }
}
