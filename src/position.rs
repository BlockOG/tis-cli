use crate::direction::Direction;

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub(crate) struct Position {
    pub(crate) x: i32,
    pub(crate) y: i32,
}

impl Position {
    pub(crate) fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub(crate) fn in_direction(&self, direction: Direction) -> Self {
        match direction {
            Direction::Up => Self {
                x: self.x,
                y: self.y + 1,
            },
            Direction::Down => Self {
                x: self.x,
                y: self.y - 1,
            },
            Direction::Left => Self {
                x: self.x - 1,
                y: self.y,
            },
            Direction::Right => Self {
                x: self.x + 1,
                y: self.y,
            },
        }
    }
}
