use std::{fs::read_to_string, ops::Range};

use ariadne::{Color, Label, Report, ReportKind, Source};
use logos::Logos;

use crate::{position::Position, utils::offset_range};

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t\r\f]+")]
enum SettingsToken {
    #[regex(r"[a-z_]+", |lex| lex.slice().to_string())]
    SpecialNode(String),

    #[regex(r"-?\d+", |lex| lex.slice().parse().ok())]
    Number(i32),

    #[token(",")]
    Comma,

    #[token(":")]
    Colon,

    #[token("acc")]
    Accumulator,

    #[token("bak")]
    Backup,
}

pub(super) enum SpecialNode {
    NumberConsoleOut,
    NumberConsoleIn,
    ConsoleOut,
    ConsoleIn,
}

impl From<String> for SpecialNode {
    fn from(value: String) -> Self {
        match value.as_str() {
            "number_console_out" => SpecialNode::NumberConsoleOut,
            "number_console_in" => SpecialNode::NumberConsoleIn,
            "console_out" => SpecialNode::ConsoleOut,
            "console_in" => SpecialNode::ConsoleIn,
            _ => panic!("Unknown special node: {}", value),
        }
    }
}

pub(super) fn parse_settings(
    start: usize,
    path: String,
    settings: &str,
) -> Option<(
    (Position, Range<usize>),
    Option<i32>,
    Option<i32>,
    Option<SpecialNode>,
)> {
    let mut settings = SettingsToken::lexer(settings);

    let mut pos = None;
    let mut accumulator = None;
    let mut backup = None;
    let mut special_node = None;

    while let Some(token) = settings.next() {
        if let Err(_) = token {
            let span = offset_range(settings.span(), start);
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
        let span = offset_range(settings.span(), start);
        match token.unwrap() {
            SettingsToken::SpecialNode(name) if special_node.is_none() => {
                special_node = Some(SpecialNode::from(name))
            }
            SettingsToken::Number(x) if pos.is_none() => {
                if let Some(Ok(SettingsToken::Comma)) = settings.next() {
                    let comma_span = offset_range(settings.span(), start);
                    if let Some(Ok(SettingsToken::Number(y))) = settings.next() {
                        pos = Some((Position::new(x, y), span.start..start + settings.span().end));
                    } else {
                        Report::build(ReportKind::Error, path.clone(), comma_span.start)
                            .with_code(0)
                            .with_message("Invalid Syntax")
                            .with_label(
                                Label::new((path.clone(), comma_span))
                                    .with_message("Here")
                                    .with_color(Color::Red),
                            )
                            .finish()
                            .print((
                                path.clone(),
                                Source::from(read_to_string(path.clone()).unwrap()),
                            ))
                            .unwrap();
                    }
                } else {
                    Report::build(ReportKind::Error, path.clone(), span.start)
                        .with_code(0)
                        .with_message("Invalid Syntax")
                        .with_label(
                            Label::new((path.clone(), span))
                                .with_message("Here")
                                .with_color(Color::Red),
                        )
                        .finish()
                        .print((
                            path.clone(),
                            Source::from(read_to_string(path.clone()).unwrap()),
                        ))
                        .unwrap();
                }
            }
            SettingsToken::Accumulator if accumulator.is_none() => {
                if let Some(Ok(SettingsToken::Colon)) = settings.next() {
                    if let Some(Ok(SettingsToken::Number(x))) = settings.next() {
                        accumulator = Some(x);
                    } else {
                        panic!("Expected number after colon");
                    }
                } else {
                    panic!("Expected colon after accumulator");
                }
            }
            SettingsToken::Backup if backup.is_none() => {
                if let Some(Ok(SettingsToken::Colon)) = settings.next() {
                    if let Some(Ok(SettingsToken::Number(x))) = settings.next() {
                        backup = Some(x);
                    } else {
                        panic!("Expected number after colon");
                    }
                } else {
                    panic!("Expected colon after backup");
                }
            }

            SettingsToken::SpecialNode(_) => {
                panic!("Special node already set");
            }
            SettingsToken::Accumulator => {
                panic!("Accumulator already set");
            }
            SettingsToken::Backup => {
                panic!("Backup already set");
            }
            SettingsToken::Number(_) => {
                Report::build(ReportKind::Error, path.clone(), span.start)
                    .with_code(1)
                    .with_message("Position already set")
                    .with_label(
                        Label::new((path.clone(), pos.unwrap().1))
                            .with_message("Already set position")
                            .with_color(Color::Blue),
                    )
                    .with_label(
                        Label::new((path.clone(), span))
                            .with_message("New position start")
                            .with_color(Color::Red),
                    )
                    .finish()
                    .print((path.clone(), Source::from(read_to_string(path).unwrap())))
                    .unwrap();
                return None;
            }
            SettingsToken::Comma => {
                panic!("Unexpected comma");
            }
            SettingsToken::Colon => {
                panic!("Unexpected colon");
            }
        }
    }

    if pos.is_none() {
        Report::build(ReportKind::Error, path.clone(), start - 1)
            .with_code(1)
            .with_message("No position provided")
            .with_label(
                Label::new((path.clone(), start - 1..start))
                    .with_message("Here")
                    .with_color(Color::Red),
            )
            .finish()
            .print((path.clone(), Source::from(read_to_string(path).unwrap())))
            .unwrap();
        None
    } else {
        Some((pos.unwrap(), accumulator, backup, special_node))
    }
}
