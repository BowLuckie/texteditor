use std::io::{self, Read};

use crossterm::{
    event::{self, Event::Key, KeyCode::Char},
    terminal::{disable_raw_mode, enable_raw_mode},
};

#[derive(Default)]

pub struct Editor {}

impl Editor {
    /// Creates a new [`Editor`].
    pub fn new() -> Self {
        Self::default()
    }

    pub fn run(&self) {
        loop {
            match event::read() {
                Ok(Key(event)) => {
                    println!("{:?} \r", event);
                    if let Char(c) = event.code
                        && c == 'q'
                    {
                        break;
                    }
                }
                Err(err) => println!("Error: {err}"),
                _ => (),
            }
        }
    }
}
