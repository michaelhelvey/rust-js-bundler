//! Parses Javascript character escape sequences.
//!
//! Note that the parser does not parse RegExp literals into their component
//! parts, and so some of the eccentricities of unicode mode and RegExp
//! character classes are omitted.
//!
//! See: https://tc39.es/ecma262/#prod-EscapeSequence

use color_eyre::{eyre::eyre, Result};
use nom::AsChar;
use std::{iter::Peekable, str::Chars};

/// Attempts to parse an octal escape sequence into a single `char`, returning
/// an Err if the sequence is out of range.  Advances the provided iterator past
/// the parsed sequence.
///
/// *Note*:  The caller is responsible for ensuring that the initial character
/// is a valid octal digit.
fn parse_octal_escape_sequence(chars: &mut Peekable<Chars>, init: char) -> Result<char> {
    let mut value = init.to_digit(8).ok_or(eyre!(
        "internal parser error: caller must check that '{}' is a valid octal",
        init
    ))?;

    'octal: for _ in 0..2 {
        let next_digit = match chars.peek() {
            Some(c) if c.is_oct_digit() => chars.next().unwrap(),
            _ => break 'octal,
        };

        value = value * 8 + next_digit.to_digit(8).unwrap();
    }

    if value > 0o377 {
        return Err(eyre!(
            "invalid octal escape sequence: out of range: {}",
            value,
        ));
    }

    // safety: we just checked that the value is in range.
    Ok(value as u8 as char)
}

/// Attempts to parse a hex escape sequence into a single `char`, returning an
/// error if the escape sequence is invalid.
fn parse_hex_escape_sequence(chars: &mut Peekable<Chars>) -> Result<char> {
    let invalid_err_msg = "Invalid hexadecimal escape sequence";

    let mut value = match chars.next() {
        Some(c) if c.is_hex_digit() => c.to_digit(16).unwrap(),
        _ => return Err(eyre!(invalid_err_msg)),
    };

    match chars.peek() {
        Some(c) if c.is_hex_digit() => {
            // safety:  we just checked the value exists and that it's a valid hex digit.
            value = value * 16 + chars.next().unwrap().to_digit(16).unwrap()
        }
        _ => return Err(eyre!(invalid_err_msg)),
    };

    std::char::from_u32(value).ok_or(eyre!(invalid_err_msg))
}

/// Attempts to parse a unicode escape sequence into a single `char`, returning
/// `Ok(char)` if the escape sequence can be parsed into a valid code point, and
/// `Err` if the escape sequence is invalid (either because it is out of range,
/// or because it is malformed).
fn parse_unicode_escape_sequence(chars: &mut Peekable<Chars>) -> Result<char> {
    let delimiter = match chars.peek() {
        Some('{') => {
            _ = chars.next();
            Some('}')
        }
        _ => None,
    };

    if let Some(trailing_delimter) = delimiter {
        let mut value = 0;

        'unicode: loop {
            let next_digit = match chars.peek() {
                Some(c) if c.is_hex_digit() => chars.next().unwrap(),
                Some(c) if *c == trailing_delimter => {
                    // Consume trailing delimiter
                    _ = chars.next();
                    break 'unicode;
                }
                _ => return Err(eyre!("Invalid hexadecimal escape sequence")),
            };

            value = value * 16 + next_digit.to_digit(16).unwrap();
        }

        if value > 0x10ffff {
            return Err(eyre!("Undefined Unicode code-point"));
        }

        std::char::from_u32(value).ok_or(eyre!("Invalid Unicode code-point"))
    } else {
        let mut value = 0;

        for _ in 0..4 {
            let next_digit = match chars.next() {
                Some(c) if c.is_hex_digit() => c,
                _ => return Err(eyre!("Invalid hexadecimal escape sequence")),
            };

            value = value * 16 + next_digit.to_digit(16).unwrap();
        }

        if value > 0x10ffff {
            return Err(eyre!("Undefined Unicode code-point"));
        }

        std::char::from_u32(value).ok_or(eyre!("Invalid Unicode code-point"))
    }
}

/// Parses a potentially multi-byte escape sequence into a single `char`, such
/// as octal escapes, unicode escapes, etc.  Returns the provided `init` value
/// as a fall through if no other matches were found.
fn parse_multi_byte_escape(chars: &mut Peekable<Chars>, init: char) -> Result<char> {
    if init.is_oct_digit() {
        return parse_octal_escape_sequence(chars, init);
    }

    if init == 'x' {
        return parse_hex_escape_sequence(chars);
    }

    if init == 'u' {
        return parse_unicode_escape_sequence(chars);
    }

    Ok(init)
}

/// Attempts to parse an iterator of characters containing an escape sequence
/// into the characters that they represent.  Assumes that the leading backslash
/// has already been consumed.
///
/// Returns:
///
/// * `Ok(Some(char))` if the next characters in the iterator are a valid
/// escape, and can be parsed into a `char`.
///
/// * Ok(None) if the next characters in the iterator are a valid escape, but
/// they should be ignored (e.g. a newline escape sequence).
///
/// * `Err` if the next characters in the iterator are an escape sequence, but
/// cannot be parsed into a `char`.
pub fn try_parse_escape(chars: &mut Peekable<Chars>) -> Result<Option<char>> {
    // Start by trying to match against a "basic" escape sequence, before trying
    // to parse multi-byte sequences like octals, unicode, control codes, etc.
    match chars.next() {
        Some('b') => Ok(Some('\u{0008}')),
        Some('f') => Ok(Some('\u{000c}')),
        Some('n') => Ok(Some('\u{000a}')),
        Some('r') => Ok(Some('\u{000d}')),
        Some('t') => Ok(Some('\u{0009}')),
        Some('v') => Ok(Some('\u{000b}')),
        Some('"') => Ok(Some('\u{0022}')),
        Some('\'') => Ok(Some('\u{0027}')),
        Some('\u{000A}') => Ok(None),
        Some('\u{000D}') => Ok(None),
        Some('\u{2028}') => Ok(None),
        Some('\u{2029}') => Ok(None),
        Some(c) => parse_multi_byte_escape(chars, c).map(Some),
        None => Err(eyre!("Unexpected EOF while parsing escape sequence")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_line_escape_sequence() {
        let mut chars = r#"n"#.chars().peekable();
        assert_eq!(try_parse_escape(&mut chars).unwrap().unwrap(), '\n');
    }

    #[test]
    fn test_non_escape_chars_interpreted_as_identity() {
        let src = r#"a"#;
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_escape(&mut chars).unwrap().unwrap(), 'a');
    }

    #[test]
    fn test_single_escape_characters() {
        // See: https://tc39.es/ecma262/#prod-SingleEscapeCharacter
        let js_single_escapes = vec![
            (r#"b"#, '\u{0008}'),
            (r#"f"#, '\u{000c}'),
            (r#"n"#, '\u{000a}'),
            (r#"r"#, '\u{000d}'),
            (r#"t"#, '\u{0009}'),
            (r#"v"#, '\u{000b}'),
            (r#"""#, '\u{0022}'),
            (r#"'"#, '\u{0027}'),
            (r#"\"#, '\u{005c}'),
        ];

        for (src, expected) in js_single_escapes {
            let mut chars = src.chars().peekable();
            assert_eq!(try_parse_escape(&mut chars).unwrap().unwrap(), expected);
        }
    }

    #[test]
    fn test_octal_escape_sequence() {
        let src = r#"0"#;
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_escape(&mut chars).unwrap().unwrap(), '\u{0000}');
    }

    #[test]
    fn test_octal_escape_sequence_out_of_range() {
        // My understanding is if an octal escape sequence is outside the valid
        // range (0-377), then in strict mode it is an error, and in sloppy mode
        // it is implementation dependent.  So in _my_ implementation, it's a
        // syntax error no matter what!
        let src = r#"777"#;
        let mut chars = src.chars().peekable();
        let result = try_parse_escape(&mut chars);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "invalid octal escape sequence: out of range: 511"
        );
    }

    #[test]
    fn test_octal_escape_sequence_does_not_eat_trailing_characters() {
        let src = r#"39"#;
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_escape(&mut chars).unwrap().unwrap(), '\u{0003}');
        assert_eq!(chars.next().unwrap(), '9');
    }

    #[test]
    fn test_hex_escape_sequence_where_no_leading_char() {
        let src = r#"x"#;
        let result = try_parse_escape(&mut src.chars().peekable());

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid hexadecimal escape sequence"
        );
    }

    #[test]
    fn test_hex_escape_sequence_where_leading_char_not_hex_digit() {
        let src = r#"xG"#;
        let result = try_parse_escape(&mut src.chars().peekable());

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid hexadecimal escape sequence"
        );
    }

    #[test]
    fn test_hex_escape_sequence_where_next_char_not_hex_digit() {
        let src = r#"xFG"#;
        let result = try_parse_escape(&mut src.chars().peekable());

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid hexadecimal escape sequence"
        );
    }

    #[test]
    fn test_valid_hex_escape_sequence() {
        let src = r#"xFF"#;
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_escape(&mut chars).unwrap().unwrap(), '\u{00ff}');
    }

    #[test]
    fn test_unicode_escape_sequence_with_braces() {
        let src = r#"u{1f600}"#;
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_escape(&mut chars).unwrap().unwrap(), '😀');
        assert_eq!(chars.next(), None)
    }

    #[test]
    fn test_unicode_escape_sequence_without_braces() {
        let src = r#"u1f600"#;
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_escape(&mut chars).unwrap().unwrap(), 'ὠ');
        assert_eq!(chars.next().unwrap(), '0');
    }

    #[test]
    fn test_unicode_escape_sequence_out_of_range() {
        let src = r#"u{1f6000}"#;
        let result = try_parse_escape(&mut src.chars().peekable());

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Undefined Unicode code-point"
        );
    }

    #[test]
    fn test_unicode_escape_sequence_invalid_chars() {
        let src = r#"u{1f6G0}"#;
        let result = try_parse_escape(&mut src.chars().peekable());

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid hexadecimal escape sequence"
        );

        let src = r#"uFFG"#;
        let result = try_parse_escape(&mut src.chars().peekable());

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid hexadecimal escape sequence"
        );
    }

    #[test]
    fn test_unicode_escape_does_not_eat_trailing_chars() {
        let src = r#"u00410"#;
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_escape(&mut chars).unwrap().unwrap(), 'A');
        assert_eq!(chars.next().unwrap(), '0');
    }
}
