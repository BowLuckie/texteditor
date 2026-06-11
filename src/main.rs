#![warn(
    clippy::all,
    clippy::pedantic,
    clippy::print_stdout,
    clippy::arithmetic_side_effects,
    clippy::as_conversions,
    clippy::integer_division,
    clippy::implicit_return
)]
#![allow(clippy::needless_return)]

mod editor;

use std::io;

use editor::Editor;

pub type IoErr = std::io::Error;
pub type UnitResult<T> = Result<(), T>;
pub type TerminalResult = io::Result<()>;

fn main() {
    let mut editor = Editor::new();
    editor.run();
}
