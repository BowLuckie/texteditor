#![allow(clippy::needless_return)]
#![allow(unused)]

use crossterm::terminal::disable_raw_mode;
use crossterm::terminal::enable_raw_mode;
use std::io::{self, Read};

mod editor;

use editor::Editor;

fn main() {
    let editor = Editor::new();
    editor.run();
}
