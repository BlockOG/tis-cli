use std::{cell::RefCell, collections::HashMap, rc::Rc};

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
    labels: HashMap<String, usize>,
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
    pub(crate) fn new(
        position: Position,
        labels: HashMap<String, usize>,
        instructions: Vec<Instruction>,
    ) -> Self {
        Self {
            position,

            up: None,
            down: None,
            left: None,
            right: None,

            labels,
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
        if self.instructions.is_empty() {
            return;
        }

        if self.give != DirectionGiving::None {
            return;
        }

        if self.ptr >= self.instructions.len() {
            self.ptr = 0;
        }

        let instruction = self.instructions[self.ptr].clone();

        let mut skip_ptr_incr = false;
        match instruction {
            Instruction::Noop => {}
            Instruction::Move(source, destination) => {
                let value = match source {
                    RegisterOrNumber::Register(register) => match self.get_value(register) {
                        None => return,
                        Some(value) => value,
                    },
                    RegisterOrNumber::Number(number) => number,
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
                let value = match source {
                    RegisterOrNumber::Register(register) => match self.get_value(register) {
                        None => return,
                        Some(value) => value,
                    },
                    RegisterOrNumber::Number(number) => number,
                };
                self.accumulator += value;
            }
            Instruction::Subtract(source) => {
                let value = match source {
                    RegisterOrNumber::Register(register) => match self.get_value(register) {
                        None => return,
                        Some(value) => value,
                    },
                    RegisterOrNumber::Number(number) => number,
                };
                self.accumulator -= value;
            }
            Instruction::Negate => {
                self.accumulator = -self.accumulator;
            }

            Instruction::Jump(label) => {
                if let Some(ptr) = self.labels.get(&label) {
                    skip_ptr_incr = true;
                    self.ptr = *ptr;
                } else {
                    panic!("Label \"{}\" not found", label);
                }
            }

            Instruction::JumpEqualZero(label) => {
                if self.accumulator.is_zero() {
                    if let Some(ptr) = self.labels.get(&label) {
                        skip_ptr_incr = true;
                        self.ptr = *ptr;
                    } else {
                        panic!("Label \"{}\" not found", label);
                    }
                }
            }
            Instruction::JumpNotZero(label) => {
                if !self.accumulator.is_zero() {
                    if let Some(ptr) = self.labels.get(&label) {
                        skip_ptr_incr = true;
                        self.ptr = *ptr;
                    } else {
                        panic!("Label \"{}\" not found", label);
                    }
                }
            }

            Instruction::JumpGreaterThanZero(label) => {
                if self.accumulator > zero() {
                    if let Some(ptr) = self.labels.get(&label) {
                        skip_ptr_incr = true;
                        self.ptr = *ptr;
                    } else {
                        panic!("Label \"{}\" not found", label);
                    }
                }
            }
            Instruction::JumpLessThanZero(label) => {
                if self.accumulator < zero() {
                    if let Some(ptr) = self.labels.get(&label) {
                        skip_ptr_incr = true;
                        self.ptr = *ptr;
                    } else {
                        panic!("Label \"{}\" not found", label);
                    }
                }
            }

            Instruction::JumpRelative(source) => {
                skip_ptr_incr = true;
                self.ptr = (self.ptr as i32
                    + match source {
                        RegisterOrNumber::Register(register) => match self.get_value(register) {
                            None => return,
                            Some(value) => value,
                        },
                        RegisterOrNumber::Number(number) => number,
                    }
                    .value() as i32)
                    .max(0) as usize;
            }
        }

        if !skip_ptr_incr {
            self.ptr += 1;
        }
    }

    fn handle_give(&mut self) {
        if self.give == DirectionGiving::None && self.give_value.is_some() {
            let register = if let Instruction::Move(_, register) = self.instructions[self.ptr] {
                register
            } else {
                unreachable!()
            };
            match register {
                Register::Direction(_) | Register::Any => self.ptr += 1,
                Register::Last => {
                    if self.last.is_some() {
                        self.ptr += 1;
                    } else {
                        return;
                    }
                }
                _ => return,
            }
            match register {
                Register::Direction(direction) => {
                    self.give = DirectionGiving::Direction(direction.clone());
                }
                Register::Any => {
                    self.give = DirectionGiving::Any;
                }
                Register::Last => {
                    self.give = DirectionGiving::Direction(self.last.unwrap());
                }
                _ => {}
            }
        }
    }

    fn post_handle_give(&mut self) -> Option<Position> {
        if let Some(giving_to) = self.giving_to {
            if self.give == DirectionGiving::Any {
                self.last = Some(giving_to);
            }
            self.give = DirectionGiving::Given;

            Some(self.position.in_direction(giving_to))
        } else {
            None
        }
    }

    fn post_post_handle_give(&mut self) {
        if self.giving_to.is_some() {
            self.give = DirectionGiving::None;
            self.giving_to = None;
        }
    }
}
