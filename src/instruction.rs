use crate::register::{Register, RegisterOrNumber};

#[derive(Debug, Clone)]
pub(crate) enum Instruction {
    Noop,
    Move(RegisterOrNumber, Register),

    // Backup register instructions
    Swap,
    Save,

    // Math instructions
    Add(RegisterOrNumber),
    Subtract(RegisterOrNumber),
    Negate,

    // Jump instructions
    Jump(usize),

    JumpEqualZero(usize),
    JumpNotZero(usize),

    JumpGreaterThanZero(usize),
    JumpLessThanZero(usize),

    JumpRelative(RegisterOrNumber),
}
