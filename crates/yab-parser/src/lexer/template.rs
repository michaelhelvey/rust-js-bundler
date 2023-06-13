use color_eyre::{eyre::eyre, Result};
use serde::Serialize;
use std::{iter::Peekable, str::Chars};

use super::escape_chars::try_parse_escape;

// Save allocating a string when we know the lexeme value already.
static TEMPLATE_LITERAL_EXPR_CLOSE: &str = "}";
static TEMPLATE_LITERAL_EXPR_OPEN: &str = "${";

#[derive(Debug, PartialEq, Serialize)]
pub struct TemplateLiteralString {
    lexeme: String,
    /// Whether the string is complete (reached a "`" or not).
    complete: bool,
}

impl TemplateLiteralString {
    pub fn new(lexeme: String, complete: bool) -> Self {
        Self { lexeme, complete }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct TemplateLiteralExprOpen {
    lexeme: &'static str,
}

#[derive(Debug, PartialEq, Serialize)]
pub struct TemplateLiteralExprClose {
    lexeme: &'static str,
}

impl Default for TemplateLiteralExprOpen {
    fn default() -> Self {
        Self {
            lexeme: TEMPLATE_LITERAL_EXPR_OPEN,
        }
    }
}

impl Default for TemplateLiteralExprClose {
    fn default() -> Self {
        Self {
            lexeme: TEMPLATE_LITERAL_EXPR_CLOSE,
        }
    }
}

/// Attempts to parse the close of a template literal expression ('}' and
/// following).  Should be used in place of parsing '}' as punctuation if in a
/// template literal context.
///
/// Return types have the same semantics as `try_parse_template_literal_start`
/// et. al.
pub fn try_parse_template_literal_expr_end(
    chars: &mut Peekable<Chars>,
) -> Result<
    Option<(
        TemplateLiteralExprClose,
        TemplateLiteralString,
        Option<TemplateLiteralExprOpen>,
    )>,
> {
    match chars.peek() {
        Some('}') => {
            _ = chars.next();
            let (string, expr_open) = parse_template_literal_string(chars)?;
            Ok(Some((
                TemplateLiteralExprClose::default(),
                string,
                expr_open,
            )))
        }
        _ => Ok(None),
    }
}

/// Once confirmed to be "in" a template string (e.g. with the leading '`' or
/// '}' consumed), parses the remaining characters of the lexeme into a template
/// literal token.
///
/// Returns:
///
/// * `Ok((TemplateLiteralString, TemplateLiteralExprOpen))` if the next
/// part of the template ends in expression opener.
///
/// * `Ok((TemplateLiteralString, None))` if the next part of the template
/// concludes the template literal.  The lexer is expected to pop off the
/// template context stack when the end of a template literal is reached, so
/// that the semantic meaning of '}' is altered.
///
/// * `Err` if the next part of the template literal could not be parsed (e.g.
/// because of an invalid escape sequence).
pub fn parse_template_literal_string(
    chars: &mut Peekable<Chars>,
) -> Result<(TemplateLiteralString, Option<TemplateLiteralExprOpen>)> {
    let mut lexeme = String::new();

    while let Some(next_char) = chars.next() {
        match next_char {
            '`' => return Ok((TemplateLiteralString::new(lexeme, true), None)),
            '$' => match chars.peek() {
                Some('{') => {
                    _ = chars.next();
                    return Ok((
                        TemplateLiteralString::new(lexeme, false),
                        Some(TemplateLiteralExprOpen::default()),
                    ));
                }
                _ => lexeme.push('$'),
            },
            '\\' => {
                // parse escape sequence
                if let Some(escaped_char) = try_parse_escape(chars)? {
                    lexeme.push(escaped_char);
                }
            }
            c => lexeme.push(c),
        }
    }

    Err(eyre!("Unexpected EOF while parsing template literal"))
}

/// Attempts to parse the start of a template literal from the top-level of the
/// lexer loop.
///
/// Returns:
///
/// * `Ok(Some((TemplateLiteralString, TemplateLiteralExprOpen)))` if the next
/// token is a template literal that ends in an expression opener (`${`).  The
/// lexer is expected to push onto a stack and then begin tokenzing the
/// expression.  The stack is used to determine whether the next '}' encountered
/// is to be interpreted as an template literal expression close token, or as a
/// punctuation token.
///
/// * `Ok(Some((TemplateLiteralString, None)))` if the next token is a template
/// literal string which is already closed, e.g. `hi there`
///
/// * `Ok(None)` if the next token is not a template literal at all.
///
/// * `Err` if the next token is a template literal but it could not be parsed
/// (e.g. due to an invalid escape sequence).
pub fn try_parse_template_literal_start(
    chars: &mut Peekable<Chars>,
) -> Result<Option<(TemplateLiteralString, Option<TemplateLiteralExprOpen>)>> {
    match chars.peek() {
        Some('`') => {
            _ = chars.next();
            parse_template_literal_string(chars).map(Some)
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_template_literal_without_expr() {
        let src = "`hi there`";
        let mut chars = src.chars().peekable();

        assert_eq!(
            try_parse_template_literal_start(&mut chars)
                .unwrap()
                .unwrap(),
            (
                TemplateLiteralString::new("hi there".to_string(), true),
                None
            )
        )
    }

    #[test]
    fn test_parse_template_literal_with_expression() {
        let src = "`hi there ${`";
        let mut chars = src.chars().peekable();

        assert_eq!(
            try_parse_template_literal_start(&mut chars)
                .unwrap()
                .unwrap(),
            (
                TemplateLiteralString::new("hi there ".to_string(), false),
                Some(TemplateLiteralExprOpen::default())
            )
        )
    }

    #[test]
    fn test_unexpected_eof_while_parsing_template_literal() {
        let src = "`hi there";
        let result = try_parse_template_literal_start(&mut src.chars().peekable());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unexpected EOF while parsing template literal"
        );
    }

    #[test]
    fn test_escape_sequences_are_parsed() {
        let src = r#"`hi ther\u0065!`"#;
        let chars = &mut src.chars().peekable();

        assert_eq!(
            try_parse_template_literal_start(chars).unwrap().unwrap(),
            (
                TemplateLiteralString::new("hi there!".to_string(), true),
                None
            )
        )
    }

    #[test]
    fn test_multi_line_template_literal() {
        let src = r#"`hi there:
        you`"#;
        let mut chars = src.chars().peekable();

        assert_eq!(
            try_parse_template_literal_start(&mut chars)
                .unwrap()
                .unwrap(),
            (
                TemplateLiteralString::new("hi there:\n        you".to_string(), true),
                None
            )
        )
    }

    #[test]
    fn test_try_parse_template_literal_expr_close() {
        let src = "} end`";
        let mut chars = src.chars().peekable();

        assert_eq!(
            try_parse_template_literal_expr_end(&mut chars)
                .unwrap()
                .unwrap(),
            (
                TemplateLiteralExprClose::default(),
                TemplateLiteralString::new(" end".to_string(), true),
                None
            )
        )
    }

    #[test]
    fn test_try_parse_template_literal_expr_with_next_expr_open() {
        let src = "} end ${`";
        let mut chars = src.chars().peekable();

        assert_eq!(
            try_parse_template_literal_expr_end(&mut chars)
                .unwrap()
                .unwrap(),
            (
                TemplateLiteralExprClose::default(),
                TemplateLiteralString::new(" end ".to_string(), false),
                Some(TemplateLiteralExprOpen::default())
            )
        )
    }

    #[test]
    fn test_expr_end_is_end_of_template_literal() {
        let src = "}`";
        let mut chars = src.chars().peekable();

        assert_eq!(
            try_parse_template_literal_expr_end(&mut chars)
                .unwrap()
                .unwrap(),
            (
                TemplateLiteralExprClose::default(),
                TemplateLiteralString::new("".to_string(), true),
                None
            )
        )
    }
}
