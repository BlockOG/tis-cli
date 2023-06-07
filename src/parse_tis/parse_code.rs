use std::{
    collections::{
        hash_map::Entry::{Occupied, Vacant},
        HashMap,
    },
    fs::read_to_string,
    ops::Range,
};

use ariadne::{Color, Label, Report, ReportKind, Source};
use logos::{Lexer, Logos};

use crate::{
    direction::Direction,
    instruction::Instruction,
    number::Number,
    register::{Register, RegisterOrNumber},
    utils::offset_range,
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
#[logos(skip r"[ \t\r\f]+|#[^\n]*")]
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

    #[regex(r"jmp[ \t\r\f]+[^ \t#\n\r\f:]+", get_label)]
    Jump(String),

    #[regex(r"jez[ \t\r\f]+[^ \t#\n\r\f:]+", get_label)]
    JumpEqualZero(String),

    #[regex(r"jnz[ \t\r\f]+[^ \t#\n\r\f:]+", get_label)]
    JumpNotZero(String),

    #[regex(r"jgz[ \t\r\f]+[^ \t#\n\r\f:]+", get_label)]
    JumpGreaterThanZero(String),

    #[regex(r"jlz[ \t\r\f]+[^ \t#\n\r\f:]+", get_label)]
    JumpLessThanZero(String),

    #[token("jro")]
    JumpRelative,

    #[regex(r"[^ \t#\n\r\f:]+:", get_label_definition)]
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

fn get_register(
    code: &mut Lexer<CodeToken>,
    span: Range<usize>,
    path: &String,
) -> Option<Register> {
    match code.next() {
        Some(Ok(CodeToken::Up)) => Some(Register::Direction(Direction::Up)),
        Some(Ok(CodeToken::Down)) => Some(Register::Direction(Direction::Down)),
        Some(Ok(CodeToken::Left)) => Some(Register::Direction(Direction::Left)),
        Some(Ok(CodeToken::Right)) => Some(Register::Direction(Direction::Right)),
        Some(Ok(CodeToken::Any)) => Some(Register::Any),
        Some(Ok(CodeToken::Last)) => Some(Register::Last),
        Some(Ok(CodeToken::Accumulator)) => Some(Register::Accumulator),
        Some(Ok(CodeToken::Nil)) => Some(Register::Nil),
        _ => {
            Report::build(ReportKind::Error, path.clone(), span.start)
                .with_code(1)
                .with_message("Expected direction or register")
                .with_label(
                    Label::new((path.clone(), span))
                        .with_message("From instruction here")
                        .with_color(Color::Blue),
                )
                .finish()
                .print((path.clone(), Source::from(read_to_string(path).unwrap())))
                .unwrap();
            None
        }
    }
}

fn get_register_or_number(
    code: &mut Lexer<CodeToken>,
    span: Range<usize>,
    path: &String,
) -> Option<RegisterOrNumber> {
    match code.next() {
        Some(Ok(CodeToken::Number(x))) => Some(RegisterOrNumber::Number(x)),
        Some(Ok(CodeToken::Up)) => Some(RegisterOrNumber::Register(Register::Direction(
            Direction::Up,
        ))),
        Some(Ok(CodeToken::Down)) => Some(RegisterOrNumber::Register(Register::Direction(
            Direction::Down,
        ))),
        Some(Ok(CodeToken::Left)) => Some(RegisterOrNumber::Register(Register::Direction(
            Direction::Left,
        ))),
        Some(Ok(CodeToken::Right)) => Some(RegisterOrNumber::Register(Register::Direction(
            Direction::Right,
        ))),
        Some(Ok(CodeToken::Any)) => Some(RegisterOrNumber::Register(Register::Any)),
        Some(Ok(CodeToken::Last)) => Some(RegisterOrNumber::Register(Register::Last)),
        Some(Ok(CodeToken::Accumulator)) => Some(RegisterOrNumber::Register(Register::Accumulator)),
        Some(Ok(CodeToken::Nil)) => Some(RegisterOrNumber::Register(Register::Nil)),
        _ => {
            Report::build(ReportKind::Error, path.clone(), span.start)
                .with_code(2)
                .with_message("Expected direction, register or number")
                .with_label(
                    Label::new((path.clone(), span))
                        .with_message("From instruction here")
                        .with_color(Color::Blue),
                )
                .finish()
                .print((path.clone(), Source::from(read_to_string(path).unwrap())))
                .unwrap();
            None
        }
    }
}

pub(super) fn parse_code(start: usize, path: String, code: &str) -> Option<Vec<Instruction>> {
    let mut code = CodeToken::lexer(code);

    let mut labels: HashMap<String, (usize, Range<usize>)> = HashMap::new();
    let mut post_processing_instructions = Vec::new();

    enum PostProcessing {
        Instruction(Instruction),

        // To be replaced with an instruction
        Jump(String, Range<usize>),

        JumpEqualZero(String, Range<usize>),
        JumpNotZero(String, Range<usize>),

        JumpGreaterThanZero(String, Range<usize>),
        JumpLessThanZero(String, Range<usize>),
    }

    impl From<Instruction> for PostProcessing {
        fn from(instruction: Instruction) -> Self {
            PostProcessing::Instruction(instruction)
        }
    }

    let mut prev_was_label = None;
    while let Some(token) = code.next() {
        prev_was_label = None;
        if let Err(_) = token {
            let span = offset_range(code.span(), start);
            Report::build(ReportKind::Error, path.clone(), span.start)
                .with_code(0)
                .with_message("Invalid Syntax")
                .with_label(
                    Label::new((path.clone(), span))
                        .with_message("Here")
                        .with_color(Color::Red),
                )
                .finish()
                .print((path.clone(), Source::from(read_to_string(path).unwrap())))
                .unwrap();
            return None;
        }
        let span = offset_range(code.span(), start);
        match token.unwrap() {
            CodeToken::Newline => continue,
            CodeToken::Label(name) => {
                match labels.entry(name) {
                    Occupied(entry) => {
                        Report::build(ReportKind::Error, path.clone(), span.start)
                            .with_code(6)
                            .with_message("Label already defined")
                            .with_label(
                                Label::new((path.clone(), entry.get().1.clone()))
                                    .with_message("Already defined label")
                                    .with_color(Color::Blue),
                            )
                            .with_label(
                                Label::new((path.clone(), span))
                                    .with_message("New label")
                                    .with_color(Color::Green),
                            )
                            .finish()
                            .print((path.clone(), Source::from(read_to_string(path).unwrap())))
                            .unwrap();
                        return None;
                    }
                    Vacant(entry) => {
                        entry.insert((post_processing_instructions.len(), span.clone()))
                    }
                };
                prev_was_label = Some(span.clone());
                continue; // A label doesn't require a newline after it
            }

            CodeToken::Noop => {
                post_processing_instructions.push(Instruction::Noop.into());
            }

            CodeToken::Move => {
                post_processing_instructions.push(
                    Instruction::Move(
                        get_register_or_number(&mut code, span.clone(), &path)?,
                        get_register(&mut code, span.clone(), &path)?,
                    )
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
                post_processing_instructions.push(
                    Instruction::Add(get_register_or_number(&mut code, span.clone(), &path)?)
                        .into(),
                );
            }
            CodeToken::Subtract => {
                post_processing_instructions.push(
                    Instruction::Subtract(get_register_or_number(&mut code, span.clone(), &path)?)
                        .into(),
                );
            }
            CodeToken::Negate => {
                post_processing_instructions.push(Instruction::Negate.into());
            }

            CodeToken::Jump(label) => {
                post_processing_instructions.push(PostProcessing::Jump(label, span.clone()));
            }

            CodeToken::JumpEqualZero(label) => {
                post_processing_instructions
                    .push(PostProcessing::JumpEqualZero(label, span.clone()));
            }
            CodeToken::JumpNotZero(label) => {
                post_processing_instructions.push(PostProcessing::JumpNotZero(label, span.clone()));
            }

            CodeToken::JumpGreaterThanZero(label) => {
                post_processing_instructions
                    .push(PostProcessing::JumpGreaterThanZero(label, span.clone()));
            }
            CodeToken::JumpLessThanZero(label) => {
                post_processing_instructions
                    .push(PostProcessing::JumpLessThanZero(label, span.clone()));
            }

            CodeToken::JumpRelative => {
                post_processing_instructions.push(
                    Instruction::JumpRelative(get_register_or_number(
                        &mut code,
                        span.clone(),
                        &path,
                    )?)
                    .into(),
                );
            }

            token => {
                let name = match token {
                    CodeToken::Accumulator => "Acc",
                    CodeToken::Any => "Any",
                    CodeToken::Last => "Last",
                    CodeToken::Nil => "Nil",
                    CodeToken::Up => "Up",
                    CodeToken::Down => "Down",
                    CodeToken::Left => "Left",
                    CodeToken::Right => "Right",
                    CodeToken::Number(_) => "Number",
                    _ => unreachable!(),
                };
                Report::build(ReportKind::Error, path.clone(), span.start)
                    .with_code(3)
                    .with_message(format!("{} can only be used as an expression", name))
                    .with_label(
                        Label::new((path.clone(), span))
                            .with_message("Here")
                            .with_color(Color::Red),
                    )
                    .finish()
                    .print((path.clone(), Source::from(read_to_string(path).unwrap())))
                    .unwrap();
                return None;
            }
        }

        match code.next() {
            Some(Ok(CodeToken::Newline)) => {}
            _ => {
                Report::build(ReportKind::Error, path.clone(), span.start)
                    .with_code(4)
                    .with_message("Expected newline after instruction")
                    .with_label(
                        Label::new((path.clone(), span))
                            .with_message("The instruction")
                            .with_color(Color::Blue),
                    )
                    .finish()
                    .print((path.clone(), Source::from(read_to_string(path).unwrap())))
                    .unwrap();
                return None;
            }
        }
    }

    if let Some(span) = prev_was_label {
        Report::build(ReportKind::Error, path.clone(), span.start)
            .with_code(5)
            .with_message("Expected anything after label")
            .with_label(
                Label::new((path.clone(), span))
                    .with_message("The label")
                    .with_color(Color::Blue),
            )
            .finish()
            .print((path.clone(), Source::from(read_to_string(path).unwrap())))
            .unwrap();
        return None;
    }

    let labels: HashMap<String, usize> = labels
        .into_iter()
        .map(|(name, (index, _span))| (name, index))
        .collect();
    let eval_label = |label: String, span: Range<usize>| {
        let res = labels.get(&label).map(|index| *index);
        if res.is_none() {
            Report::build(ReportKind::Error, path.clone(), span.start)
                .with_code(7)
                .with_message("Label not found")
                .with_label(
                    Label::new((path.clone(), span))
                        .with_message("Label usage")
                        .with_color(Color::Blue),
                )
                .finish()
                .print((
                    path.clone(),
                    Source::from(read_to_string(path.clone()).unwrap()),
                ))
                .unwrap();
        }
        res
    };

    post_processing_instructions
        .into_iter()
        .map(|instruction| {
            Some(match instruction {
                PostProcessing::Instruction(instruction) => instruction,

                PostProcessing::Jump(label, span) => Instruction::Jump(eval_label(label, span)?),

                PostProcessing::JumpEqualZero(label, span) => {
                    Instruction::JumpEqualZero(eval_label(label, span)?)
                }
                PostProcessing::JumpNotZero(label, span) => {
                    Instruction::JumpNotZero(eval_label(label, span)?)
                }

                PostProcessing::JumpGreaterThanZero(label, span) => {
                    Instruction::JumpGreaterThanZero(eval_label(label, span)?)
                }
                PostProcessing::JumpLessThanZero(label, span) => {
                    Instruction::JumpLessThanZero(eval_label(label, span)?)
                }
            })
        })
        .collect()
}
