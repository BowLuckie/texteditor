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
#![deny(clippy::todo)]

mod editor;
use core::panic;

use editor::Editor;

fn main() {
    panic!("");
    let mut editor = Editor::new().unwrap();
    editor.run();
}
