use crate::{direction::Direction, number::Number};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Register {
    Accumulator,
    // Bak,
    Nil,

    // Directions
    Direction(Direction),

    // Special
    Any,
    Last,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RegisterOrNumber {
    Register(Register),
    Number(Number),
}
