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

pub(super) fn parse_code(code: &str) -> Vec<Instruction> {
    let mut code = CodeToken::lexer(code).peekable();

    let mut labels = HashMap::new();
    let mut post_processing_instructions = Vec::new();

    enum PostProcessing {
        Instruction(Instruction),

        // To be replaced with an instruction
        Jump(String),

        JumpEqualZero(String),
        JumpNotZero(String),

        JumpGreaterThanZero(String),
        JumpLessThanZero(String),
    }

    impl From<Instruction> for PostProcessing {
        fn from(instruction: Instruction) -> Self {
            PostProcessing::Instruction(instruction)
        }
    }

    while let Some(token) = code.next() {
        match token.expect("Failed to parse code") {
            CodeToken::Newline => continue,
            CodeToken::Label(name) => {
                match labels.entry(name) {
                    Occupied(entry) => panic!("Label already defined: {}", entry.key()),
                    Vacant(entry) => entry.insert(post_processing_instructions.len()),
                };
                if code.peek().is_none() {
                    panic!("Label must be followed by code or a newline");
                }
                continue; // A label doesn't require a newline after it
            }

            CodeToken::Noop => {
                post_processing_instructions.push(Instruction::Noop.into());
            }

            CodeToken::Move => {
                post_processing_instructions.push(
                    Instruction::Move(get_register_or_number(&mut code), get_register(&mut code))
                        .into(),
                );
            }

            CodeToken::Swap => {
                post_processing_instructions.push(Instruction::Swap.into());
            }
            CodeToken::Save => {
                post_processing_instructions.push(Instruction::Save.into());
            }

            CodeToken::Add => {
                post_processing_instructions
                    .push(Instruction::Add(get_register_or_number(&mut code)).into());
            }
            CodeToken::Subtract => {
                post_processing_instructions
                    .push(Instruction::Subtract(get_register_or_number(&mut code)).into());
            }
            CodeToken::Negate => {
                post_processing_instructions.push(Instruction::Negate.into());
            }

            CodeToken::Jump(label) => {
                post_processing_instructions.push(PostProcessing::Jump(label));
            }

            CodeToken::JumpEqualZero(label) => {
                post_processing_instructions.push(PostProcessing::JumpEqualZero(label));
            }
            CodeToken::JumpNotZero(label) => {
                post_processing_instructions.push(PostProcessing::JumpNotZero(label));
            }

            CodeToken::JumpGreaterThanZero(label) => {
                post_processing_instructions.push(PostProcessing::JumpGreaterThanZero(label));
            }
            CodeToken::JumpLessThanZero(label) => {
                post_processing_instructions.push(PostProcessing::JumpLessThanZero(label));
            }

            CodeToken::JumpRelative => {
                post_processing_instructions
                    .push(Instruction::JumpRelative(get_register_or_number(&mut code)).into());
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

    let eval_label = |label: String| {
        *labels
            .get(&label)
            .expect(format!("Label \"{}\" not found", label).as_str())
    };

    post_processing_instructions
        .into_iter()
        .map(|instruction| match instruction {
            PostProcessing::Instruction(instruction) => instruction,

            PostProcessing::Jump(label) => Instruction::Jump(eval_label(label)),

            PostProcessing::JumpEqualZero(label) => Instruction::JumpEqualZero(eval_label(label)),
            PostProcessing::JumpNotZero(label) => Instruction::JumpNotZero(eval_label(label)),

            PostProcessing::JumpGreaterThanZero(label) => {
                Instruction::JumpGreaterThanZero(eval_label(label))
            }
            PostProcessing::JumpLessThanZero(label) => {
                Instruction::JumpLessThanZero(eval_label(label))
            }
        })
        .collect()
}
