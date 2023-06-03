use enum_iterator::Sequence;

#[derive(Debug, Clone, Copy, Sequence, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Direction {
    Up,
    Left,
    Right,
    Down,
}

impl Direction {
    pub(crate) fn opposite(&self) -> Self {
        match self {
            Self::Up => Self::Down,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
            Self::Down => Self::Up,
        }
    }
}
