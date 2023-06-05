use std::{cell::RefCell, io, rc::Rc};

use enum_iterator::all;

use crate::{direction::Direction, number::Number, position::Position};

use super::{DirectionGiving, Node};

pub(crate) struct NumberConsoleOutNode {
    position: Position,

    // Directions
    up: Option<Rc<RefCell<dyn Node>>>,
    down: Option<Rc<RefCell<dyn Node>>>,
    left: Option<Rc<RefCell<dyn Node>>>,
    right: Option<Rc<RefCell<dyn Node>>>,
}

impl NumberConsoleOutNode {
    pub(crate) fn new(position: Position) -> Self {
        Self {
            position,

            up: None,
            down: None,
            left: None,
            right: None,
        }
    }
}

impl Node for NumberConsoleOutNode {
    fn position(&self) -> Position {
        self.position
    }

    fn set_dir(&mut self, dir: Direction, node: Rc<RefCell<(dyn Node + 'static)>>) {
        match dir {
            Direction::Up => self.up = Some(node),
            Direction::Down => self.down = Some(node),
            Direction::Left => self.left = Some(node),
            Direction::Right => self.right = Some(node),
        }
    }

    fn give(&self) -> &DirectionGiving {
        &DirectionGiving::None
    }

    fn giving_to(&self) -> Option<Direction> {
        None
    }

    fn set_giving_to(&mut self, _direction: Direction) {}

    fn give_value(&mut self) -> &mut Option<Number> {
        unreachable!("NumberConsoleOutNode does not give values");
    }

    fn tick(&mut self) {
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
                        }
                        Some(prev_direction) => {
                            node.set_giving_to(prev_direction.min(direction.opposite()));
                        }
                    },
                    DirectionGiving::Direction(giving_direction) => {
                        if giving_direction == &direction.opposite() {
                            node.set_giving_to(direction.opposite());
                        }
                    }
                    DirectionGiving::Given => {
                        println!("{}", node.give_value().take().unwrap().value());
                    }
                }
            }
        }
    }

    fn handle_give(&mut self) {}

    fn post_handle_give(&mut self) -> Option<Position> {
        None
    }

    fn post_post_handle_give(&mut self) {}
}

pub(crate) struct NumberConsoleInNode {
    position: Position,

    // Directions
    up: Option<Rc<RefCell<dyn Node>>>,
    down: Option<Rc<RefCell<dyn Node>>>,
    left: Option<Rc<RefCell<dyn Node>>>,
    right: Option<Rc<RefCell<dyn Node>>>,

    // Direction transmition
    give: DirectionGiving,
    giving_to: Option<Direction>,
    give_value: Option<Number>,
}

impl NumberConsoleInNode {
    pub(crate) fn new(position: Position) -> Self {
        Self {
            position,

            up: None,
            down: None,
            left: None,
            right: None,

            give: DirectionGiving::Any,
            giving_to: None,
            give_value: None,
        }
    }
}

impl Node for NumberConsoleInNode {
    fn position(&self) -> Position {
        self.position
    }

    fn set_dir(&mut self, dir: Direction, node: Rc<RefCell<(dyn Node + 'static)>>) {
        match dir {
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
        let mut input = String::new();
        loop {
            io::stdin().read_line(&mut input).unwrap();
            match input.trim().parse::<Number>() {
                Ok(value) => {
                    self.give_value = Some(value);
                    break;
                }
                Err(_) => {
                    println!("Please enter a valid integer");
                    input.clear();
                }
            }
        }

        &mut self.give_value
    }

    fn tick(&mut self) {}

    fn handle_give(&mut self) {}

    fn post_handle_give(&mut self) -> Option<Position> {
        let giving_to = self.giving_to?;
        self.give = DirectionGiving::Given;
        Some(self.position.in_direction(giving_to))
    }

    fn post_post_handle_give(&mut self) {
        self.give = DirectionGiving::Any;
        self.giving_to = None;
    }
}
