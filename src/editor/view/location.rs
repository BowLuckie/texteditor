use crate::editor::terminal::Position;

#[derive(Copy, Clone, Default)]
pub struct Location {
    pub x: usize,
    pub y: usize,
}

impl From<Location> for Position {
    fn from(loc: Location) -> Self {
        return Self {
            col: loc.x,
            row: loc.y,
        };
    }
}

impl Location {
    pub const fn diff(&self, other: &Self) -> Self {
        return Self {
            x: self.x.saturating_sub(other.x),
            y: self.y.saturating_sub(other.y),
        };
    }
}
