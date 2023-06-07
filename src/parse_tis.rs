mod parse_code;
mod parse_settings;

use std::fs::read_to_string;

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

pub(crate) fn parse(tis: &mut TIS, path: String) -> Result<(), Option<String>> {
    let Ok(code) = read_to_string(&path) else {
        return Err(Some("Couldn't read file".to_owned()));
    };

    if let Some(mut start) = code.find("@") {
        for node_code in (code.to_lowercase() + "\n").split("@").skip(1) {
            let (settings, code) = node_code
                .split_once("\n")
                .ok_or("There has to be a newline separator between nodes".to_owned())?;

            start += 1;
            let ((pos, pos_span), accumulator, backup, special_node) =
                parse_settings(start, path.clone(), settings).ok_or(None)?;

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

            start += settings.len() + 1;
            let instructions = parse_code(start, path.clone(), code).ok_or(None)?;
            let mut node = InstructionNode::new(pos, instructions);
            if let Some(accumulator) = accumulator {
                node = node.with_accumulator(accumulator.into());
            }
            if let Some(backup) = backup {
                node = node.with_backup(backup.into());
            }

            tis.add_node(node);
            start += code.len();
        }
    }

    Ok(())
}
