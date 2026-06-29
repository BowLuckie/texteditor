use std::time::{Duration, Instant};

use crossterm::style::{Attribute, Color, SetForegroundColor};

use crate::editor::{terminal::Terminal, uicomponent::UiComponent};

use super::terminal::{IoResult, Size};

const MESSAGE_LIFETIME: Duration = Duration::new(5, 0);

// message color types

const RED: SetForegroundColor = SetForegroundColor(Color::Rgb {
    r: 204,
    g: 36,
    b: 29,
});

const RESET: Attribute = Attribute::Reset;

#[derive(Debug, Default, Clone)]
pub struct MessageBar {
    current_message: Message,
    needs_redraw: bool,
    cleared_after_expiry: bool,
}

#[derive(Debug, Clone)]
pub struct Message {
    message: String,
    created_at: Instant,
}

impl Default for Message {
    fn default() -> Self {
        return Self {
            message: String::default(),
            created_at: Instant::now(),
        };
    }
}

impl Message {
    fn is_expired(&self) -> bool {
        return Instant::now().duration_since(self.created_at) > MESSAGE_LIFETIME;
    }
}

impl MessageBar {
    pub fn update_message(&mut self, new_msg: &str) {
        if new_msg != self.current_message.message {
            self.current_message.message = new_msg.into();
            self.current_message.created_at = Instant::now();
            self.cleared_after_expiry = false;
            self.mark_redraw(true);
        }
    }

    pub fn update_error(&mut self, new_err: &str) {
        let err_string = format!("{RED}{new_err}{RESET}");
        self.update_message(&err_string);
    }
}

impl UiComponent for MessageBar {
    fn mark_redraw(&mut self, value: bool) {
        self.needs_redraw = value;
    }

    fn needs_redraw(&self) -> bool {
        return self.needs_redraw
            || (!self.cleared_after_expiry && self.current_message.is_expired());
    }

    fn set_size(&mut self, _size: Size) {}

    fn draw(&mut self, origin_y: usize) -> IoResult {
        let mut message: &str = &self.current_message.message;
        if self.current_message.is_expired() {
            self.cleared_after_expiry = true;
            message = "";
        }

        Terminal::print_row_with_attribute(origin_y, message, Attribute::Bold)?;

        return Ok(());
    }
}
