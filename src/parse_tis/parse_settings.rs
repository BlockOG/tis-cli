use logos::Logos;

use crate::position::Position;

#[derive(Logos, Debug, PartialEq)]
#[logos(skip r"[ \t]+")]
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
    settings: &str,
) -> (Position, Option<i32>, Option<i32>, Option<SpecialNode>) {
    let mut settings = SettingsToken::lexer(settings);

    let mut pos = None;
    let mut accumulator = None;
    let mut backup = None;
    let mut special_node = None;

    while let Some(token) = settings.next() {
        match token.expect("Failed to parse settings") {
            SettingsToken::SpecialNode(name) if special_node.is_none() => {
                special_node = Some(SpecialNode::from(name))
            }
            SettingsToken::Number(x) if pos.is_none() => {
                if let Some(Ok(SettingsToken::Comma)) = settings.next() {
                    if let Some(Ok(SettingsToken::Number(y))) = settings.next() {
                        pos = Some(Position::new(x, y));
                    } else {
                        panic!("Expected number after comma");
                    }
                } else {
                    panic!("Expected comma after number");
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
                panic!("Expected a single position");
            }
            SettingsToken::Comma => {
                panic!("Unexpected comma");
            }
            SettingsToken::Colon => {
                panic!("Unexpected colon");
            }
        }
    }

    (
        pos.expect("No position provided"),
        accumulator,
        backup,
        special_node,
    )
}
