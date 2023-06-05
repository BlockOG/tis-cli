mod parse_code;
mod parse_settings;

use crate::{
    node::{
        console_node::{ConsoleInNode, ConsoleOutNode},
        instruction_node::InstructionNode,
        number_console_node::{NumberConsoleInNode, NumberConsoleOutNode},
    },
    parse_tis::{
        parse_code::parse_code,
        parse_settings::{parse_settings, SpecialNode},
    },
    tis::TIS,
};

pub(crate) fn parse(tis: &mut TIS, code: String) {
    for node_code in (code.to_lowercase() + "\n").split("@").skip(1) {
        let node_code = node_code.replace("\r\n", "\n");
        let (settings, code) = node_code
            .split_once("\n")
            .expect("There has to be a newline separator between nodes");

        let (pos, accumulator, backup, special_node) = parse_settings(settings);

        if let Some(special_node) = special_node {
            if accumulator.is_some() {
                panic!("Special nodes don't have accumulators");
            }
            if backup.is_some() {
                panic!("Special nodes don't have backups");
            }

            match special_node {
                SpecialNode::NumberConsoleOut => tis.add_node(NumberConsoleOutNode::new(pos)),
                SpecialNode::NumberConsoleIn => tis.add_node(NumberConsoleInNode::new(pos)),
                SpecialNode::ConsoleOut => tis.add_node(ConsoleOutNode::new(pos)),
                SpecialNode::ConsoleIn => tis.add_node(ConsoleInNode::new(pos)),
            }

            continue;
        }

        let instructions = parse_code(code);
        let mut node = InstructionNode::new(pos, instructions);
        if let Some(accumulator) = accumulator {
            node = node.with_accumulator(accumulator.into());
        }
        if let Some(backup) = backup {
            node = node.with_backup(backup.into());
        }

        tis.add_node(node);
    }
}
