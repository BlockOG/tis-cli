use std::collections::HashMap;

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

pub(super) fn parse_code(code: &str) -> (HashMap<String, usize>, Vec<Instruction>) {
    let mut code = CodeToken::lexer(code);

    let mut labels = HashMap::new();
    let mut instructions = Vec::new();

    while let Some(token) = code.next() {
        match token.expect("Failed to parse code") {
            CodeToken::Newline => {}
            CodeToken::Label(name) => {
                if labels.contains_key(&name) {
                    panic!("Label already defined: {}", name);
                }

                labels.insert(name, instructions.len());
            }

            CodeToken::Noop => {
                instructions.push(Instruction::Noop);
                match code.next() {
                    Some(Ok(CodeToken::Newline)) => {}
                    _ => panic!("Expected newline"),
                }
            }

            CodeToken::Move => {
                let source;
                match code.next() {
                    Some(Ok(CodeToken::Number(x))) => source = RegisterOrNumber::Number(x),
                    Some(Ok(CodeToken::Up)) => {
                        source = RegisterOrNumber::Register(Register::Direction(Direction::Up))
                    }
                    Some(Ok(CodeToken::Down)) => {
                        source = RegisterOrNumber::Register(Register::Direction(Direction::Down))
                    }
                    Some(Ok(CodeToken::Left)) => {
                        source = RegisterOrNumber::Register(Register::Direction(Direction::Left))
                    }
                    Some(Ok(CodeToken::Right)) => {
                        source = RegisterOrNumber::Register(Register::Direction(Direction::Right))
                    }
                    Some(Ok(CodeToken::Any)) => source = RegisterOrNumber::Register(Register::Any),
                    Some(Ok(CodeToken::Last)) => {
                        source = RegisterOrNumber::Register(Register::Last)
                    }
                    Some(Ok(CodeToken::Accumulator)) => {
                        source = RegisterOrNumber::Register(Register::Accumulator)
                    }
                    Some(Ok(CodeToken::Nil)) => source = RegisterOrNumber::Register(Register::Nil),
                    _ => panic!("Expected number, direction or register"),
                }

                match code.next() {
                    Some(Ok(CodeToken::Up)) => instructions.push(Instruction::Move(
                        source,
                        Register::Direction(Direction::Up),
                    )),
                    Some(Ok(CodeToken::Down)) => instructions.push(Instruction::Move(
                        source,
                        Register::Direction(Direction::Down),
                    )),
                    Some(Ok(CodeToken::Left)) => instructions.push(Instruction::Move(
                        source,
                        Register::Direction(Direction::Left),
                    )),
                    Some(Ok(CodeToken::Right)) => instructions.push(Instruction::Move(
                        source,
                        Register::Direction(Direction::Right),
                    )),
                    Some(Ok(CodeToken::Any)) => {
                        instructions.push(Instruction::Move(source, Register::Any))
                    }
                    Some(Ok(CodeToken::Last)) => {
                        instructions.push(Instruction::Move(source, Register::Last))
                    }
                    Some(Ok(CodeToken::Accumulator)) => {
                        instructions.push(Instruction::Move(source, Register::Accumulator))
                    }
                    Some(Ok(CodeToken::Nil)) => {
                        instructions.push(Instruction::Move(source, Register::Nil))
                    }
                    _ => panic!("Expected direction or register"),
                }

                match code.next() {
                    Some(Ok(CodeToken::Newline)) => {}
                    _ => panic!("Expected newline"),
                }
            }

            CodeToken::Swap => {
                instructions.push(Instruction::Swap);
                match code.next() {
                    Some(Ok(CodeToken::Newline)) => {}
                    _ => panic!("Expected newline"),
                }
            }
            CodeToken::Save => {
                instructions.push(Instruction::Save);
                match code.next() {
                    Some(Ok(CodeToken::Newline)) => {}
                    _ => panic!("Expected newline"),
                }
            }

            CodeToken::Add => {
                match code.next() {
                    Some(Ok(CodeToken::Number(x))) => {
                        instructions.push(Instruction::Add(RegisterOrNumber::Number(x)))
                    }
                    Some(Ok(CodeToken::Up)) => instructions.push(Instruction::Add(
                        RegisterOrNumber::Register(Register::Direction(Direction::Up)),
                    )),
                    Some(Ok(CodeToken::Down)) => instructions.push(Instruction::Add(
                        RegisterOrNumber::Register(Register::Direction(Direction::Down)),
                    )),
                    Some(Ok(CodeToken::Left)) => instructions.push(Instruction::Add(
                        RegisterOrNumber::Register(Register::Direction(Direction::Left)),
                    )),
                    Some(Ok(CodeToken::Right)) => instructions.push(Instruction::Add(
                        RegisterOrNumber::Register(Register::Direction(Direction::Right)),
                    )),
                    Some(Ok(CodeToken::Any)) => instructions
                        .push(Instruction::Add(RegisterOrNumber::Register(Register::Any))),
                    Some(Ok(CodeToken::Last)) => instructions
                        .push(Instruction::Add(RegisterOrNumber::Register(Register::Last))),
                    Some(Ok(CodeToken::Accumulator)) => instructions.push(Instruction::Add(
                        RegisterOrNumber::Register(Register::Accumulator),
                    )),
                    Some(Ok(CodeToken::Nil)) => instructions
                        .push(Instruction::Add(RegisterOrNumber::Register(Register::Nil))),
                    _ => panic!("Expected number, direction or register"),
                }
                match code.next() {
                    Some(Ok(CodeToken::Newline)) => {}
                    _ => panic!("Expected newline"),
                }
            }
            CodeToken::Subtract => {
                match code.next() {
                    Some(Ok(CodeToken::Number(x))) => {
                        instructions.push(Instruction::Subtract(RegisterOrNumber::Number(x)))
                    }
                    Some(Ok(CodeToken::Up)) => instructions.push(Instruction::Subtract(
                        RegisterOrNumber::Register(Register::Direction(Direction::Up)),
                    )),
                    Some(Ok(CodeToken::Down)) => instructions.push(Instruction::Subtract(
                        RegisterOrNumber::Register(Register::Direction(Direction::Down)),
                    )),
                    Some(Ok(CodeToken::Left)) => instructions.push(Instruction::Subtract(
                        RegisterOrNumber::Register(Register::Direction(Direction::Left)),
                    )),
                    Some(Ok(CodeToken::Right)) => instructions.push(Instruction::Subtract(
                        RegisterOrNumber::Register(Register::Direction(Direction::Right)),
                    )),
                    Some(Ok(CodeToken::Any)) => instructions.push(Instruction::Subtract(
                        RegisterOrNumber::Register(Register::Any),
                    )),
                    Some(Ok(CodeToken::Last)) => instructions.push(Instruction::Subtract(
                        RegisterOrNumber::Register(Register::Last),
                    )),
                    Some(Ok(CodeToken::Accumulator)) => instructions.push(Instruction::Subtract(
                        RegisterOrNumber::Register(Register::Accumulator),
                    )),
                    Some(Ok(CodeToken::Nil)) => instructions.push(Instruction::Subtract(
                        RegisterOrNumber::Register(Register::Nil),
                    )),
                    _ => panic!("Expected number, direction or register"),
                }
                match code.next() {
                    Some(Ok(CodeToken::Newline)) => {}
                    _ => panic!("Expected newline"),
                }
            }
            CodeToken::Negate => {
                instructions.push(Instruction::Negate);
                match code.next() {
                    Some(Ok(CodeToken::Newline)) => {}
                    _ => panic!("Expected newline"),
                }
            }

            CodeToken::Jump(label) => {
                instructions.push(Instruction::Jump(label));

                match code.next() {
                    Some(Ok(CodeToken::Newline)) => {}
                    _ => panic!("Expected newline"),
                }
            }

            CodeToken::JumpEqualZero(label) => {
                instructions.push(Instruction::JumpEqualZero(label));

                match code.next() {
                    Some(Ok(CodeToken::Newline)) => {}
                    _ => panic!("Expected newline"),
                }
            }
            CodeToken::JumpNotZero(label) => {
                instructions.push(Instruction::JumpNotZero(label));

                match code.next() {
                    Some(Ok(CodeToken::Newline)) => {}
                    _ => panic!("Expected newline"),
                }
            }

            CodeToken::JumpGreaterThanZero(label) => {
                instructions.push(Instruction::JumpGreaterThanZero(label));

                match code.next() {
                    Some(Ok(CodeToken::Newline)) => {}
                    _ => panic!("Expected newline"),
                }
            }
            CodeToken::JumpLessThanZero(label) => {
                instructions.push(Instruction::JumpLessThanZero(label));

                match code.next() {
                    Some(Ok(CodeToken::Newline)) => {}
                    _ => panic!("Expected newline"),
                }
            }

            CodeToken::JumpRelative => {
                match code.next() {
                    Some(Ok(CodeToken::Number(x))) => {
                        instructions.push(Instruction::JumpRelative(RegisterOrNumber::Number(x)))
                    }
                    Some(Ok(CodeToken::Up)) => instructions.push(Instruction::JumpRelative(
                        RegisterOrNumber::Register(Register::Direction(Direction::Up)),
                    )),
                    Some(Ok(CodeToken::Down)) => instructions.push(Instruction::JumpRelative(
                        RegisterOrNumber::Register(Register::Direction(Direction::Down)),
                    )),
                    Some(Ok(CodeToken::Left)) => instructions.push(Instruction::JumpRelative(
                        RegisterOrNumber::Register(Register::Direction(Direction::Left)),
                    )),
                    Some(Ok(CodeToken::Right)) => instructions.push(Instruction::JumpRelative(
                        RegisterOrNumber::Register(Register::Direction(Direction::Right)),
                    )),
                    Some(Ok(CodeToken::Any)) => instructions.push(Instruction::JumpRelative(
                        RegisterOrNumber::Register(Register::Any),
                    )),
                    Some(Ok(CodeToken::Last)) => instructions.push(Instruction::JumpRelative(
                        RegisterOrNumber::Register(Register::Last),
                    )),
                    Some(Ok(CodeToken::Accumulator)) => {
                        instructions.push(Instruction::JumpRelative(RegisterOrNumber::Register(
                            Register::Accumulator,
                        )))
                    }
                    Some(Ok(CodeToken::Nil)) => instructions.push(Instruction::JumpRelative(
                        RegisterOrNumber::Register(Register::Nil),
                    )),
                    _ => panic!("Expected number, direction or register"),
                }

                match code.next() {
                    Some(Ok(CodeToken::Newline)) => {}
                    _ => panic!("Expected newline"),
                }
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
    }

    (labels, instructions)
}
