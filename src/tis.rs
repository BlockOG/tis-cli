use std::{cell::RefCell, collections::HashMap, rc::Rc};

use enum_iterator::all;

use crate::{direction::Direction, node::Node, position::Position};

pub(crate) struct TIS {
    nodes: HashMap<Position, Rc<RefCell<dyn Node>>>,
}

impl TIS {
    pub(crate) fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub(crate) fn add_node<T>(&mut self, node: T)
    where
        T: Node + 'static,
    {
        let node = Rc::new(RefCell::new(node));
        if self.nodes.contains_key(&node.borrow().position()) {
            panic!(
                "Node already exists at position {:?}",
                node.borrow().position()
            );
        }

        for dir in all::<Direction>() {
            let dir_pos = node.borrow().position().in_direction(dir);
            self.nodes.get(&dir_pos).map(|dir_node| {
                dir_node.borrow_mut().set_dir(dir.opposite(), node.clone());
                node.borrow_mut().set_dir(dir, dir_node.clone());
            });
        }
        let pos = node.borrow().position();
        self.nodes.insert(pos, node);
    }

    pub(crate) fn tick(&mut self) {
        for node in self.nodes.values() {
            node.borrow_mut().tick();
        }

        for node in self.nodes.values() {
            node.borrow_mut().handle_give();
        }

        for node in self.nodes.values() {
            let pos = node.borrow_mut().post_handle_give();
            if let Some(pos) = pos {
                self.nodes.get(&pos).map(|n| n.borrow_mut().tick());
                node.borrow_mut().post_post_handle_give();
            }
        }
    }
}
