use super::{code_iter::CodeIter, utils::is_line_terminator};
use serde::Serialize;

#[derive(Debug, PartialEq, Serialize)]
pub struct Comment {
    pub value: CommentType,
}

impl Comment {
    pub fn new(value: CommentType) -> Self {
        Self { value }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub enum CommentType {
    Block(String),
    Line(String),
    Hashbang(String),
}

/// Parses a line comment, assuming that the leading '//' has already been
/// consumed.
fn parse_line_comment(chars: &mut CodeIter) -> CommentType {
    let lexeme = chars
        .take_while(|c| !is_line_terminator(*c))
        .collect::<String>();

    CommentType::Line(lexeme)
}

/// Parses a a block comment, assuming that the leading '/*' has already been
/// consumed.
fn parse_block_comment(chars: &mut CodeIter) -> CommentType {
    let mut lexeme = String::new();

    while let Some(next_char) = chars.next() {
        if next_char == '*' && chars.peek() == Some(&'/') {
            chars.next();
            break;
        }

        lexeme.push(next_char);
    }

    CommentType::Block(lexeme)
}

/// Attempts to parse the following characters of the iterator into a Javascript
/// comment token (either a line comment or a block comment), returning None if
/// the next token is not a comment.
pub fn try_parse_comment(chars: &mut CodeIter) -> Option<Comment> {
    // question: this doesn't copy the underlying memory we are iterator over,
    // right?  I'm just copying a pointer and some state?
    match (chars.peek(), chars.peek_forward(1)) {
        (Some('/'), Some('/')) => {
            // feature(iter_advance_by) waiting room ResidentSleeper
            // Currently using 1.70.0
            for _ in 0..2 {
                _ = chars.next();
            }
            Some(Comment::new(parse_line_comment(chars)))
        }
        (Some('/'), Some('*')) => {
            for _ in 0..2 {
                _ = chars.next();
            }
            Some(Comment::new(parse_block_comment(chars)))
        }
        _ => None,
    }
}

pub fn try_parse_hashbang_comment(chars: &mut CodeIter) -> Option<Comment> {
    match (chars.peek(), chars.peek_forward(1)) {
        (Some('#'), Some('!')) => {
            for _ in 0..2 {
                _ = chars.next();
            }
            let lexeme = chars
                .take_while(|c| !is_line_terminator(*c))
                .collect::<String>();
            Some(Comment::new(CommentType::Hashbang(lexeme)))
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::code_iter::IntoCodeIterator;

    use super::*;

    #[test]
    fn test_parse_line_comment() {
        let mut chars = "// this is a comment\nA".into_code_iterator("script.js".to_string());
        let comment = try_parse_comment(&mut chars).unwrap();
        assert_eq!(
            comment,
            Comment {
                value: CommentType::Line(" this is a comment".to_string())
            }
        );
        assert_eq!(chars.next().unwrap(), 'A');
    }

    #[test]
    fn test_parse_block_comment() {
        let src = r#"/* this is a comment */
        A"#;
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_comment(&mut chars).unwrap(),
            Comment {
                value: CommentType::Block(" this is a comment ".to_string())
            }
        );
        assert_eq!(chars.next().unwrap(), '\n');
    }

    #[test]
    fn test_parse_hashbang_comment() {
        let src = r#"#!/usr/bin/env node"#;
        let mut chars = src.into_code_iterator("script.js".to_string());

        assert_eq!(
            try_parse_hashbang_comment(&mut chars).unwrap(),
            Comment {
                value: CommentType::Hashbang("/usr/bin/env node".to_string())
            }
        );
    }
}
