#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division
)]
#![allow(clippy::needless_return)]
#![allow(unused)]

use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use std::io::{self, Read};

mod editor;

use editor::Editor;

pub type IoErr = std::io::Error;
pub type UnitResult<T> = Result<(), T>;
pub type TerminalResult = io::Result<()>;

fn main() {
    let mut editor = Editor::new();
    editor.run();
}
