#![allow(dead_code)]
use std::{iter::Peekable, str::Chars};

use color_eyre::{eyre::eyre, Result};

use super::utils::is_line_terminator;

/// Represents a regex literal token.  Since we're not actually parsing the
/// regex, or evaluating it, we don't need to parse the pattern, just the
/// pattern + the flags so that we can compile the literal into a function call
/// later if we want.
#[derive(Debug, PartialEq)]
pub struct RegexLiteral {
    pattern: String,
    flags: String,
}

/// Parses a regex pattern, assuming that the leading '/' has been consumed.
/// Consumes the trailing '/' and returns the string in between as a pattern.
/// Does not parse escape sequences, as the runtime RegEx engine will handle
/// that.
fn parse_regex_pattern(chars: &mut Peekable<Chars>) -> Result<String> {
    let mut lexeme = String::new();
    while let Some(next_char) = chars.next() {
        match next_char {
            '/' => return Ok(lexeme),
            c if is_line_terminator(c) => {
                return Err(eyre!(
                    "Unexpected line terminator while parsing regular expression"
                ))
            }
            c => lexeme.push(c),
        }
    }

    Err(eyre!("Unterminated regex literal"))
}

fn parse_regex_flags(chars: &mut Peekable<Chars>) -> Result<String> {
    let mut lexeme = String::new();

    while let Some(next_char) = chars.next() {
        match next_char {
            'g' | 'i' | 'm' | 's' | 'u' | 'y' => {
                lexeme.push(next_char);
            }
            c if c.is_whitespace() => return Ok(lexeme),
            ';' => return Ok(lexeme),
            c => return Err(eyre!("Invalid regular expression flag '{}'", c)),
        }
    }

    Ok(lexeme)
}

/// Attempts to parse a regex literal (e.g. "/foo/g").
///
/// Returns:
///
/// * `Ok(Some(RegexLiteral))` if a regex literal was parsed.
///
/// * `Ok(None)` if the next characters are not a regex literal.
///
/// * `Err` if an error occurred while parsing (e.g. if an invalid character or
/// escape is encountered).
///
/// Note: this function is fairly naive about the difference between regex
/// literals and comments, (e.g. /{pattern/ vs "//"}), so it assumes that the
/// lexer tries to parse comments higher up in the loop.
pub fn try_parse_regex_literal(chars: &mut Peekable<Chars>) -> Result<Option<RegexLiteral>> {
    match chars.peek() {
        Some('/') => {
            _ = chars.next();
            let pattern = parse_regex_pattern(chars)?;
            let flags = parse_regex_flags(chars)?;

            Ok(Some(RegexLiteral { pattern, flags }))
        }
        _ => return Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_try_parse_regex_literal() {
        let mut chars = "/foo/g".chars().peekable();
        let result = try_parse_regex_literal(&mut chars).unwrap().unwrap();
        assert_eq!(
            result,
            RegexLiteral {
                pattern: "foo".to_string(),
                flags: "g".to_string(),
            }
        );
    }

    #[test]
    fn test_regex_without_flags() {
        let mut chars = "/foo/".chars().peekable();
        let result = try_parse_regex_literal(&mut chars).unwrap().unwrap();
        assert_eq!(
            result,
            RegexLiteral {
                pattern: "foo".to_string(),
                flags: "".to_string(),
            }
        );
    }

    #[test]
    fn test_regex_with_invalid_flags() {
        let mut chars = "/foo/z".chars().peekable();
        let result = try_parse_regex_literal(&mut chars);

        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid regular expression flag 'z'"
        );
    }

    #[test]
    fn test_regex_with_unexpected_line_break() {
        let mut chars = "/foo\n/z".chars().peekable();
        let result = try_parse_regex_literal(&mut chars);

        assert_eq!(
            result.unwrap_err().to_string(),
            "Unexpected line terminator while parsing regular expression"
        );
    }
}
