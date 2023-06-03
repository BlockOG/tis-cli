pub(crate) mod console_node;
pub(crate) mod instruction_node;
pub(crate) mod number_console_node;

use std::{cell::RefCell, rc::Rc};

use crate::{direction::Direction, number::Number, position::Position};

pub(crate) trait Node {
    fn position(&self) -> Position;
    fn set_dir(&mut self, dir: Direction, node: Rc<RefCell<dyn Node>>);

    fn give(&self) -> &DirectionGiving;
    fn giving_to(&self) -> Option<Direction>;
    fn set_giving_to(&mut self, direction: Direction);
    fn give_value(&mut self) -> &mut Option<Number>;

    fn tick(&mut self);
    fn handle_give(&mut self);
    fn post_handle_give(&mut self) -> Option<Position>;
    fn post_post_handle_give(&mut self);
}

#[derive(Debug, PartialEq, Eq)]
pub(crate) enum DirectionGiving {
    None,
    Any,
    Direction(Direction),
    Given,
}
