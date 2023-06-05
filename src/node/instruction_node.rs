use std::{cell::RefCell, rc::Rc};

use enum_iterator::all;
use num_traits::{zero, Zero};

use crate::{
    direction::Direction,
    instruction::Instruction,
    number::Number,
    position::Position,
    register::{Register, RegisterOrNumber},
};

use super::{DirectionGiving, Node};

pub(crate) struct InstructionNode {
    position: Position,

    // Directions
    up: Option<Rc<RefCell<dyn Node>>>,
    down: Option<Rc<RefCell<dyn Node>>>,
    left: Option<Rc<RefCell<dyn Node>>>,
    right: Option<Rc<RefCell<dyn Node>>>,

    // Instructions
    instructions: Vec<Instruction>,
    ptr: usize,

    // Registers
    accumulator: Number,
    backup: Number,
    last: Option<Direction>,

    // Direction transmition
    give: DirectionGiving,
    give_value: Option<Number>,
    giving_to: Option<Direction>,
}

impl InstructionNode {
    pub(crate) fn new(position: Position, instructions: Vec<Instruction>) -> Self {
        Self {
            position,

            up: None,
            down: None,
            left: None,
            right: None,

            instructions,
            ptr: 0,

            accumulator: Number::new(),
            backup: Number::new(),
            last: None,

            give: DirectionGiving::None,
            give_value: None,
            giving_to: None,
        }
    }

    pub(crate) fn with_accumulator(mut self, accumulator: Number) -> Self {
        self.accumulator = accumulator;
        self
    }

    pub(crate) fn with_backup(mut self, backup: Number) -> Self {
        self.backup = backup;
        self
    }

    fn get_value(&mut self, register: Register) -> Option<Number> {
        match register {
            Register::Accumulator => Some(self.accumulator),
            Register::Nil => Some(zero()),
            Register::Direction(direction) => {
                if let Some(node) = match direction {
                    Direction::Up => self.up.as_mut(),
                    Direction::Down => self.down.as_mut(),
                    Direction::Left => self.left.as_mut(),
                    Direction::Right => self.right.as_mut(),
                } {
                    let mut node = node.borrow_mut();
                    match node.give() {
                        DirectionGiving::None => None,
                        DirectionGiving::Any => match node.giving_to() {
                            None => {
                                node.set_giving_to(direction.opposite());
                                None
                            }
                            Some(prev_direction) => {
                                node.set_giving_to(prev_direction.min(direction.opposite()));
                                None
                            }
                        },
                        DirectionGiving::Direction(giving_direction) => {
                            if giving_direction == &direction.opposite() {
                                node.set_giving_to(direction.opposite());
                            }
                            None
                        }
                        DirectionGiving::Given => node.give_value().take(),
                    }
                } else {
                    None
                }
            }
            Register::Any => {
                for direction in all::<Direction>() {
                    if let Some(node) = match direction {
                        Direction::Up => self.up.as_mut(),
                        Direction::Down => self.down.as_mut(),
                        Direction::Left => self.left.as_mut(),
                        Direction::Right => self.right.as_mut(),
                    } {
                        let mut node = node.borrow_mut();
                        match node.give() {
                            DirectionGiving::None => {}
                            DirectionGiving::Any => match node.giving_to() {
                                None => {
                                    node.set_giving_to(direction.opposite());
                                    return None;
                                }
                                Some(prev_direction) => {
                                    node.set_giving_to(prev_direction.min(direction.opposite()));
                                    return None;
                                }
                            },
                            DirectionGiving::Direction(giving_direction) => {
                                if giving_direction == &direction.opposite() {
                                    node.set_giving_to(direction.opposite());
                                    return None;
                                }
                            }
                            DirectionGiving::Given => {
                                return node.give_value().take();
                            }
                        }
                    }
                }
                None
            }
            Register::Last => match self.last {
                None => Some(zero()),
                Some(direction) => self.get_value(Register::Direction(direction)),
            },
        }
    }

    fn get_from_register_or_number(
        &mut self,
        register_or_number: RegisterOrNumber,
    ) -> Option<Number> {
        match register_or_number {
            RegisterOrNumber::Register(register) => self.get_value(register),
            RegisterOrNumber::Number(number) => Some(number),
        }
    }

    fn set_value(&mut self, register: Register, value: Number) -> bool {
        match register {
            Register::Accumulator => {
                self.accumulator = value;
                false
            }
            Register::Nil => false,
            Register::Direction(_) | Register::Any => {
                self.give_value = Some(value);
                true
            }
            Register::Last => {
                if self.last.is_some() {
                    self.give_value = Some(value);
                    true
                } else {
                    false
                }
            }
        }
    }
}

impl Node for InstructionNode {
    fn position(&self) -> Position {
        self.position
    }

    fn set_dir(&mut self, direction: Direction, node: Rc<RefCell<dyn Node>>) {
        match direction {
            Direction::Up => self.up = Some(node),
            Direction::Down => self.down = Some(node),
            Direction::Left => self.left = Some(node),
            Direction::Right => self.right = Some(node),
        }
    }

    fn give(&self) -> &DirectionGiving {
        &self.give
    }

    fn giving_to(&self) -> Option<Direction> {
        self.giving_to
    }

    fn set_giving_to(&mut self, direction: Direction) {
        self.giving_to = Some(direction);
    }

    fn give_value(&mut self) -> &mut Option<Number> {
        &mut self.give_value
    }

    fn tick(&mut self) {
        if self.instructions.is_empty() || self.give != DirectionGiving::None {
            return;
        }

        if self.ptr >= self.instructions.len() {
            self.ptr = 0;
        }

        let instruction = self.instructions[self.ptr].clone();

        let mut skip_ptr_incr = false;
        let mut jump = |ptr: usize| {
            skip_ptr_incr = true;
            self.ptr = ptr;
        };

        match instruction {
            Instruction::Move(source, destination) => {
                let Some(value) = self.get_from_register_or_number(source) else {
                    return
                };
                skip_ptr_incr = self.set_value(destination, value);
            }

            Instruction::Swap => {
                (self.accumulator, self.backup) = (self.backup, self.accumulator);
            }
            Instruction::Save => {
                self.backup = self.accumulator;
            }

            Instruction::Add(source) => {
                let Some(value) = self.get_from_register_or_number(source) else {
                    return
                };
                self.accumulator += value;
            }
            Instruction::Subtract(source) => {
                let Some(value) = self.get_from_register_or_number(source) else {
                    return
                };
                self.accumulator -= value;
            }
            Instruction::Negate => {
                self.accumulator = -self.accumulator;
            }

            Instruction::Jump(ptr) => jump(ptr),

            Instruction::JumpEqualZero(ptr) if self.accumulator.is_zero() => jump(ptr),
            Instruction::JumpNotZero(ptr) if !self.accumulator.is_zero() => jump(ptr),

            Instruction::JumpGreaterThanZero(ptr) if self.accumulator > zero() => jump(ptr),
            Instruction::JumpLessThanZero(ptr) if self.accumulator < zero() => jump(ptr),

            Instruction::JumpRelative(source) => {
                skip_ptr_incr = true;
                self.ptr = (self.ptr as i32
                    + match self.get_from_register_or_number(source) {
                        Some(number) => number,
                        None => return,
                    }
                    .value() as i32)
                    .max(0) as usize;
            }

            _ => {}
        }

        if !skip_ptr_incr {
            self.ptr += 1;
        }
    }

    fn handle_give(&mut self) {
        if self.give == DirectionGiving::None && self.give_value.is_some() {
            let Instruction::Move(_, register) = self.instructions[self.ptr] else {
                unreachable!("What on earth did you do? Report this to https://github.com/BlockOG/tis-cli/issues")
            };
            match register {
                Register::Direction(_) | Register::Any => self.ptr += 1,
                Register::Last if self.last.is_some() => self.ptr += 1,
                _ => return,
            }
            self.give = match register {
                Register::Direction(direction) => DirectionGiving::Direction(direction.clone()),
                Register::Any => DirectionGiving::Any,
                Register::Last => DirectionGiving::Direction(self.last.unwrap()),
                _ => unreachable!(),
            };
        }
    }

    fn post_handle_give(&mut self) -> Option<Position> {
        let giving_to = self.giving_to?;
        if self.give == DirectionGiving::Any {
            self.last = Some(giving_to);
        }
        self.give = DirectionGiving::Given;

        Some(self.position.in_direction(giving_to))
    }

    fn post_post_handle_give(&mut self) {
        self.give = DirectionGiving::None;
        self.giving_to = None;
    }
}
