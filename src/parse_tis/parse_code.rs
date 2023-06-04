use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    iter::Peekable,
};

use logos::{Lexer, Logos};

use crate::{
    direction::Direction,
    instruction::Instruction,
    number::Number,
    register::{Register, RegisterOrNumber},
};

fn get_label(lex: &mut Lexer<CodeToken>) -> String {
    lex.slice()
        .chars()
        .skip(3)
        .skip_while(|&c| c == ' ' || c == '\t')
        .collect()
}

fn get_label_definition(lex: &mut Lexer<CodeToken>) -> String {
    lex.slice().chars().take_while(|&c| c != ':').collect()
}

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t]+|#[^\n]*")]
enum CodeToken {
    #[token("nop")]
    Noop,

    #[token("mov")]
    Move,

    #[token("swp")]
    Swap,

    #[token("sav")]
    Save,

    #[token("add")]
    Add,

    #[token("sub")]
    Subtract,

    #[token("neg")]
    Negate,

    #[regex("jmp[ \t]+[^ \t#\n:]+", get_label)]
    Jump(String),

    #[regex("jez[ \t]+[^ \t#\n:]+", get_label)]
    JumpEqualZero(String),

    #[regex("jnz[ \t]+[^ \t#\n:]+", get_label)]
    JumpNotZero(String),

    #[regex("jgz[ \t]+[^ \t#\n:]+", get_label)]
    JumpGreaterThanZero(String),

    #[regex("jlz[ \t]+[^ \t#\n:]+", get_label)]
    JumpLessThanZero(String),

    #[token("jro")]
    JumpRelative,

    #[regex(r"[^ \t#\n:]+:", get_label_definition)]
    Label(String),

    #[token("\n")]
    Newline,

    #[regex(r"-?\d+", |lex| lex.slice().parse().ok())]
    Number(Number),

    #[token("up")]
    Up,

    #[token("down")]
    Down,

    #[token("left")]
    Left,

    #[token("right")]
    Right,

    #[token("any")]
    Any,

    #[token("last")]
    Last,

    #[token("acc")]
    Accumulator,

    #[token("nil")]
    Nil,
}

fn get_register(code: &mut Peekable<Lexer<CodeToken>>) -> Register {
    match code.next() {
        Some(Ok(CodeToken::Up)) => Register::Direction(Direction::Up),
        Some(Ok(CodeToken::Down)) => Register::Direction(Direction::Down),
        Some(Ok(CodeToken::Left)) => Register::Direction(Direction::Left),
        Some(Ok(CodeToken::Right)) => Register::Direction(Direction::Right),
        Some(Ok(CodeToken::Any)) => Register::Any,
        Some(Ok(CodeToken::Last)) => Register::Last,
        Some(Ok(CodeToken::Accumulator)) => Register::Accumulator,
        Some(Ok(CodeToken::Nil)) => Register::Nil,
        _ => panic!("Expected direction or register"),
    }
}

fn get_register_or_number(code: &mut Peekable<Lexer<CodeToken>>) -> RegisterOrNumber {
    match code.next() {
        Some(Ok(CodeToken::Number(x))) => RegisterOrNumber::Number(x),
        Some(Ok(CodeToken::Up)) => RegisterOrNumber::Register(Register::Direction(Direction::Up)),
        Some(Ok(CodeToken::Down)) => {
            RegisterOrNumber::Register(Register::Direction(Direction::Down))
        }
        Some(Ok(CodeToken::Left)) => {
            RegisterOrNumber::Register(Register::Direction(Direction::Left))
        }
        Some(Ok(CodeToken::Right)) => {
            RegisterOrNumber::Register(Register::Direction(Direction::Right))
        }
        Some(Ok(CodeToken::Any)) => RegisterOrNumber::Register(Register::Any),
        Some(Ok(CodeToken::Last)) => RegisterOrNumber::Register(Register::Last),
        Some(Ok(CodeToken::Accumulator)) => RegisterOrNumber::Register(Register::Accumulator),
        Some(Ok(CodeToken::Nil)) => RegisterOrNumber::Register(Register::Nil),
        _ => panic!("Expected number, direction or register"),
    }
}

pub(super) fn parse_code(code: &str) -> (HashMap<String, usize>, Vec<Instruction>) {
    let mut code = CodeToken::lexer(code).peekable();

    let mut labels = HashMap::new();
    let mut instructions = Vec::new();

    while let Some(token) = code.next() {
        match token.expect("Failed to parse code") {
            CodeToken::Newline => continue,
            CodeToken::Label(name) => {
                match labels.entry(name) {
                    Occupied(entry) => panic!("Label already defined: {}", entry.key()),
                    Vacant(entry) => entry.insert(instructions.len()),
                };
                if code.peek().is_none() {
                    panic!("Label must be followed by code or a newline");
                }
                continue; // A label doesn't require a newline after it
            }

            CodeToken::Noop => {
                instructions.push(Instruction::Noop);
            }

            CodeToken::Move => {
                instructions.push(Instruction::Move(
                    get_register_or_number(&mut code),
                    get_register(&mut code),
                ));
            }

            CodeToken::Swap => {
                instructions.push(Instruction::Swap);
            }
            CodeToken::Save => {
                instructions.push(Instruction::Save);
            }

            CodeToken::Add => {
                instructions.push(Instruction::Add(get_register_or_number(&mut code)));
            }
            CodeToken::Subtract => {
                instructions.push(Instruction::Subtract(get_register_or_number(&mut code)));
            }
            CodeToken::Negate => {
                instructions.push(Instruction::Negate);
            }

            CodeToken::Jump(label) => {
                instructions.push(Instruction::Jump(label));
            }

            CodeToken::JumpEqualZero(label) => {
                instructions.push(Instruction::JumpEqualZero(label));
            }
            CodeToken::JumpNotZero(label) => {
                instructions.push(Instruction::JumpNotZero(label));
            }

            CodeToken::JumpGreaterThanZero(label) => {
                instructions.push(Instruction::JumpGreaterThanZero(label));
            }
            CodeToken::JumpLessThanZero(label) => {
                instructions.push(Instruction::JumpLessThanZero(label));
            }

            CodeToken::JumpRelative => {
                instructions.push(Instruction::JumpRelative(get_register_or_number(&mut code)));
            }

            CodeToken::Accumulator => panic!("Accumulator can only be used in expressions"),
            CodeToken::Any => panic!("Any can only be used in expressions"),
            CodeToken::Last => panic!("Last can only be used in expressions"),
            CodeToken::Nil => panic!("Nil can only be used in expressions"),
            CodeToken::Up => panic!("Up can only be used in expressions"),
            CodeToken::Down => panic!("Down can only be used in expressions"),
            CodeToken::Left => panic!("Left can only be used in expressions"),
            CodeToken::Right => panic!("Right can only be used in expressions"),
            CodeToken::Number(_) => panic!("Number can only be used in expressions"),
        }

        match code.next() {
            Some(Ok(CodeToken::Newline)) => {}
            _ => panic!("Expected newline"),
        }
    }

    (labels, instructions)
}
