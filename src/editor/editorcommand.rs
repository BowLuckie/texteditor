use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use std::convert::TryFrom;

use super::terminal::Size;

pub enum Direction {
    PageUp,
    PageDown,
    Home,
    End,
    Up,
    Left,
    Right,
    Down,
}
pub enum EditorCommand {
    Move(Direction),
    Resize(Size),
    Quit,
    Insert(char),
    Del,
    Backspace,
    Tab,
    Enter,
    Save,
}

impl TryFrom<&Event> for EditorCommand {
    type Error = String;
    fn try_from(event: &Event) -> Result<Self, Self::Error> {
        #![allow(clippy::as_conversions)]
        return match event {
            Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match code {
                KeyCode::Char('q') if modifiers == &KeyModifiers::CONTROL => {
                    return Ok(Self::Quit);
                }

                KeyCode::Char(c)
                    if [KeyModifiers::SHIFT, KeyModifiers::NONE].contains(modifiers) =>
                {
                    Ok(Self::Insert(*c))
                }

                KeyCode::Backspace => Ok(Self::Backspace),
                KeyCode::Delete => Ok(Self::Del),

                KeyCode::Tab => Ok(Self::Tab),
                KeyCode::Enter => Ok(Self::Enter),

                KeyCode::Char(char @ ('j' | 'k' | 'l' | 'h'))
                    if modifiers == &KeyModifiers::ALT =>
                {
                    Ok(Self::Move(handle_vi_move(*char)))
                }

                KeyCode::Char('s') if modifiers == &KeyModifiers::CONTROL => Ok(Self::Save),

                KeyCode::Up => Ok(Self::Move(Direction::Up)),
                KeyCode::Down => Ok(Self::Move(Direction::Down)),
                KeyCode::Left => Ok(Self::Move(Direction::Left)),
                KeyCode::Right => Ok(Self::Move(Direction::Right)),
                KeyCode::PageDown => Ok(Self::Move(Direction::PageDown)),
                KeyCode::PageUp => Ok(Self::Move(Direction::PageUp)),
                KeyCode::Home => Ok(Self::Move(Direction::Home)),
                KeyCode::End => Ok(Self::Move(Direction::End)),
                _ => return Err(format!("Key Code not supported: {code:?}")),
            },
            Event::Resize(width_u16, height_u16) => {
                let height = *height_u16 as usize;
                let width = *width_u16 as usize;
                return Ok(Self::Resize(Size { height, width }));
            }
            _ => return Err(format!("Event not supported: {event:?}")),
        };
    }
}

fn handle_vi_move(direction: char) -> Direction {
    assert!(&['h', 'j', 'k', 'l'].contains(&direction));
    return match direction {
        'k' => Direction::Up,
        'j' => Direction::Down,
        'h' => Direction::Left,
        'l' => Direction::Right,
        _ => unreachable!(),
    };
}
