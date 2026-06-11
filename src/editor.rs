use crossterm::event::{
    self,
    Event::{self, Key},
    KeyCode::{self, Char, End, Home, PageDown, PageUp},
    KeyEvent,
    KeyEventKind::Press,
    KeyModifiers,
};
use event::KeyCode::{Down, Left, Right, Up};

use crate::{
    TerminalResult,
    editor::terminal::{Position, Size},
};

mod terminal;
use terminal::Terminal as Term;

mod view;
use view::View;

/// represents a complete editor.
#[derive(Default)]
pub struct Editor {
    should_quit: bool,
    cursor_location: Position,
    view: View,
}

impl Editor {
    /// Creates a new [`Editor`].
    pub fn new() -> Self {
        return Self::default();
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
            self.evaluate_event(&event)?;
        }
        return Ok(());
    }

    fn evaluate_event(&mut self, event: &Event) -> TerminalResult {
        if let Key(KeyEvent {
            code, modifiers, ..
        }) = event
            && let Char('q') = code
            && modifiers == &KeyModifiers::CONTROL
        {
            self.should_quit = true;
        }

        if let Key(KeyEvent {
            code, kind: Press, ..
        }) = event
            && let Up | Down | Left | Right | PageUp | PageDown | Home | End = *code
        {
            self.directional_move(*code)?;
        }

        return Ok(());
    }

    fn refresh_screen(&self) -> TerminalResult {
        Term::hide_cursor()?;
        Term::move_cursor(Position::default())?;
        if self.should_quit {
            Term::clear_screen()?;
            Term::print("Goodbye.\r\n")?;
        } else {
            self.view.render()?;
            Term::move_cursor(self.cursor_location)?;
        }
        Term::show_cursor()?;
        Term::flush()?;
        return Ok(());
    }

    fn move_cursor_location(&mut self, position: Position) {
        self.cursor_location = position;
    }

    fn directional_move(&mut self, direction: KeyCode) -> TerminalResult {
        #![allow(clippy::arithmetic_side_effects)]
        let Position { x, y } = self.cursor_location;
        let Size { height, width } = Term::size()?;
        let (max_x, max_y) = (width.saturating_sub(1), height.saturating_sub(1));

        let new_pos = match direction {
            Up if y > 0 => Position { x, y: y - 1 },
            Down if y < max_y => Position { x, y: y + 1 },
            Left if x > 0 => Position { x: x - 1, y },
            Right if x < max_x => Position { x: x + 1, y },
            PageUp => Position { x, y: 0 },
            PageDown => Position { x, y: max_y },
            Home => Position { x: 0, y },
            End => Position { x: max_x, y },
            _ => return Ok(()),
        };

        self.move_cursor_location(new_pos);
        return Ok(());
    }
}
