use crate::editor::terminal::{IoResult, Size};

pub trait UiComponent {
    fn mark_redraw(&mut self, value: bool);

    fn needs_redraw(&self) -> bool;

    fn resize(&mut self, size: Size) {
        self.set_size(size);
        self.mark_redraw(true);
    }

    fn set_size(&mut self, size: Size);

    fn render(&mut self, origin_y: usize) {
        if !self.needs_redraw() {
            return;
        }

        let result = self.draw(origin_y);
        if result.is_ok() {
            self.mark_redraw(true);
        } else {
            #[cfg(debug_assertions)]
            panic!(
                "Could not render component! {}",
                result.expect_err("unreachable")
            )
        }
    }

    fn draw(&mut self, origin_y: usize) -> IoResult;
}
