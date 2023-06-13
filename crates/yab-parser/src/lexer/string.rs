use std::{iter::Peekable, str::Chars};

use miette::{miette, Result};
use serde::Serialize;

use super::escape_chars::try_parse_escape;

/// Represents a string literal token, with delimiters stripped.
#[derive(Debug, Serialize, PartialEq)]
pub struct StringLiteral {
    lexeme: String,
}

impl StringLiteral {
    /// Creates a new empty string literal.
    pub fn new(lexeme: String) -> Self {
        Self { lexeme }
    }
}

impl From<String> for StringLiteral {
    fn from(value: String) -> Self {
        Self { lexeme: value }
    }
}

impl From<&str> for StringLiteral {
    fn from(value: &str) -> Self {
        Self {
            lexeme: value.to_string(),
        }
    }
}

/// Attempts to parse a string out of an iterator of characters.
///
/// Returns:
///
/// * `Ok(Some(StringLiteral))` if a string was parsed.  The iterator will have
/// been advanced to the end of the string (including the delimter).
///
/// * `Ok(None)` if no string was parsed.  The iterator will be unchanged.
///
/// * `Err` if an error occurred while parsing the string (e.g. an invalid
/// escape character or unexpected EOF).
pub fn try_parse_string(chars: &mut Peekable<Chars>) -> Result<Option<StringLiteral>> {
    let mut lexeme = String::new();

    let delimeter = match chars.peek() {
        Some('\'') | Some('"') => chars.next().unwrap(),
        _ => return Ok(None),
    };

    let mut found_end = false;
    'string: while let Some(next_char) = chars.next() {
        if next_char == delimeter {
            found_end = true;
            break 'string;
        }

        if super::utils::is_line_terminator(next_char) {
            return Err(miette!(
                "Unexpected line terminator while parsing string literal"
            ));
        }

        if next_char == '\\' {
            if let Some(escaped_char) = try_parse_escape(chars)? {
                lexeme.push(escaped_char);
            }
        } else {
            lexeme.push(next_char);
        }
    }

    if !found_end {
        return Err(miette!("Unexpected EOF while parsing string literal"));
    }

    Ok(Some(lexeme.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_double_quote_delimted_string() {
        let src = r#""hello world""#;
        let mut chars = src.chars().peekable();

        let result = try_parse_string(&mut chars).unwrap().unwrap();

        assert_eq!(result, StringLiteral::from("hello world"));
        assert_eq!(chars.next(), None);
    }

    #[test]
    fn test_parse_single_quoted_string() {
        let src = r#"'hello world'"#;
        let mut chars = src.chars().peekable();

        let result = try_parse_string(&mut chars).unwrap().unwrap();

        assert_eq!(result, StringLiteral::from("hello world"));
        assert_eq!(chars.next(), None);
    }

    #[test]
    fn test_empty_string_returns_none() {
        let src = r#""#;
        let mut chars = src.chars().peekable();

        let result = try_parse_string(&mut chars).unwrap();

        assert_eq!(result, None);
        assert_eq!(chars.next(), None);
    }

    #[test]
    fn test_invalid_delimiter_returns_none() {
        let src = r#"hello world"#;
        let mut chars = src.chars().peekable();

        let result = try_parse_string(&mut chars).unwrap();

        assert_eq!(result, None);
        assert_eq!(chars.next(), Some('h'));
    }

    #[test]
    fn test_unexpected_line_terminator_returns_err() {
        let src = r#""hello
        world""#;
        let mut chars = src.chars().peekable();

        let result = try_parse_string(&mut chars);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unexpected line terminator while parsing string literal"
        );
    }

    #[test]
    fn test_unexpected_eof_returns_err() {
        let src = r#""hello world"#;
        let mut chars = src.chars().peekable();

        let result = try_parse_string(&mut chars);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unexpected EOF while parsing string literal"
        );
    }

    #[test]
    fn test_escape_sequences_are_parsed() {
        let src = r#""hello\nworld \u{1f600}""#;
        let mut chars = src.chars().peekable();
        assert_eq!(
            try_parse_string(&mut chars).unwrap().unwrap(),
            "hello\nworld 😀".into()
        );
    }

    #[test]
    fn test_escape_sequences_eat_appropriate_leading_and_trailing_chars() {
        let src = r#""\u0041\u0042C""#;
        let mut chars = src.chars().peekable();

        assert_eq!(try_parse_string(&mut chars).unwrap().unwrap(), "ABC".into());
    }

    #[test]
    fn test_escaped_line_character() {
        let src = r#""hello\
 world""#;
        let mut chars = src.chars().peekable();

        assert_eq!(
            try_parse_string(&mut chars).unwrap().unwrap(),
            "hello world".into()
        );
    }
}
