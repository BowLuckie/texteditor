use crossterm::event::{
    Event,
    KeyCode::{self, Char},
    KeyEvent, KeyModifiers,
};
use std::convert::TryFrom;

use crate::editor::terminal::Size;

#[derive(Debug, Clone, Copy)]
pub enum Move {
    PageUp,
    PageDown,
    StartOfLine,
    EndOfLine,
    Up,
    Left,
    Right,
    Down,
}

impl TryFrom<&KeyEvent> for Move {
    type Error = String;
    fn try_from(event: &KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;
        if modifiers != &KeyModifiers::NONE {
            return Err(format!(
                "Unsupported key code {code:?} or modifier {modifiers:?}"
            ));
        }
        return match code {
            KeyCode::Up => Ok(Self::Up),
            KeyCode::Down => Ok(Self::Down),
            KeyCode::Left => Ok(Self::Left),
            KeyCode::Right => Ok(Self::Right),
            KeyCode::PageDown => Ok(Self::PageDown),
            KeyCode::PageUp => Ok(Self::PageUp),
            KeyCode::Home => Ok(Self::StartOfLine),
            KeyCode::End => Ok(Self::EndOfLine),
            KeyCode::Char(ch @ ('h' | 'j' | 'k' | 'l')) if modifiers == &KeyModifiers::CONTROL => {
                Ok(handle_vi_move(*ch))
            }
            _ => Err(format!("unsupported code: {code:?}")),
        };
    }
}

fn handle_vi_move(direction: char) -> Move {
    assert!(&['h', 'j', 'k', 'l'].contains(&direction));
    return match direction {
        'k' => Move::Up,
        'j' => Move::Down,
        'h' => Move::Left,
        'l' => Move::Right,
        _ => unreachable!(),
    };
}

#[derive(Debug, Clone, Copy)]
pub enum Edit {
    Insert(char),
    NewLine,
    Delete,
    Backspace,
}

impl TryFrom<&KeyEvent> for Edit {
    type Error = String;
    fn try_from(event: &KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;
        if let KeyCode::Char(ch) = code
            && [KeyModifiers::NONE, KeyModifiers::SHIFT].contains(modifiers)
        {
            return Ok(Self::Insert(*ch));
        }
        if modifiers != &KeyModifiers::NONE {
            return Err(format!(
                "Unsupported key code {code:?} or modifier {modifiers:?}"
            ));
        }
        return match code {
            KeyCode::Tab => Ok(Self::Insert('\t')),
            KeyCode::Enter => Ok(Self::NewLine),
            KeyCode::Delete => Ok(Self::Delete),
            KeyCode::Backspace => Ok(Self::Backspace),
            _ => Err(format!(
                "Unsupported key code {code:?} or modifier {modifiers:?}"
            )),
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub enum System {
    Save,
    Resize(Size),
    Quit,
}

impl TryFrom<&KeyEvent> for System {
    type Error = String;
    fn try_from(event: &KeyEvent) -> Result<Self, Self::Error> {
        let KeyEvent {
            code, modifiers, ..
        } = event;
        if modifiers != &KeyModifiers::CONTROL {
            return Err(format!(
                "Unsupported key code {code:?} or modifier {modifiers:?}"
            ));
        }

        return match code {
            Char('q') => Ok(Self::Quit),
            Char('s') => Ok(Self::Save),
            _ => Err(format!("Unsupported CONTROL+{code:?} combination")),
        };
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Command {
    Move(Move),
    Edit(Edit),
    System(System),
}

impl TryFrom<&Event> for Command {
    type Error = String;
    fn try_from(event: &Event) -> Result<Self, Self::Error> {
        #![allow(clippy::implicit_return)]
        return match event {
            Event::Key(ke) => Edit::try_from(ke)
                .map(Command::Edit)
                .or_else(|_| Move::try_from(ke).map(Command::Move))
                .or_else(|_| System::try_from(ke).map(Command::System)),
            Event::Resize(width, height) => Ok(sys_resize_from(*width, *height)),
            _ => Err(format!("Event not supported: {event:?}")),
        };
    }
}

fn sys_resize_from(width: u16, height: u16) -> Command {
    #![allow(clippy::as_conversions)]
    return Command::System(System::Resize(Size {
        height: height as usize,
        width: width as usize,
    }));
}
