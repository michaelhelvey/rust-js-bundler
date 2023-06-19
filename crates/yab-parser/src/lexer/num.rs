use miette::{miette, IntoDiagnostic, Result};
use nom::AsChar;
use serde::Serialize;

use super::code_iter::CodeIter;

#[derive(Debug, PartialEq, Serialize)]
pub struct NumberLiteral {
    pub value: NumberLiteralValue,
}

impl NumberLiteral {
    pub fn new(value: NumberLiteralValue) -> Self {
        Self { value }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub enum NumberLiteralValue {
    Primitive(f64),
    BigInt(BigIntStorage),
}

#[derive(Debug, PartialEq)]
pub struct BigIntStorage {
    pub value: num_bigint::BigInt,
    pub lexeme: String,
}

impl Serialize for BigIntStorage {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.lexeme)
    }
}

impl From<f64> for NumberLiteralValue {
    fn from(value: f64) -> Self {
        Self::Primitive(value)
    }
}

impl From<i32> for NumberLiteralValue {
    fn from(value: i32) -> Self {
        Self::Primitive(value as f64)
    }
}

enum Sign {
    Positive,
    Negative,
}

impl std::fmt::Display for Sign {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Positive => "+",
                Self::Negative => "-",
            }
        )
    }
}

impl From<Option<char>> for Sign {
    fn from(s: Option<char>) -> Self {
        match s {
            Some('-') => Self::Negative,
            _ => Self::Positive,
        }
    }
}

impl Sign {
    fn apply_f64(&self, value: f64) -> f64 {
        match self {
            Self::Positive => value,
            Self::Negative => -value,
        }
    }

    fn apply_i64(&self, value: i64) -> i64 {
        match self {
            Self::Positive => value,
            Self::Negative => -(value),
        }
    }
}

fn is_numeric_separator(c: char) -> bool {
    c == '_'
}

// Attempts to parse the exponent of a scientific notation number.  Assumes that
// the leading "e" has not yet been consumed.
fn parse_scientific_exponent(chars: &mut CodeIter) -> Result<i64> {
    let mut lexeme = String::new();
    _ = chars.next(); // trailing 'e'

    let sign = match chars.peek() {
        Some('+') | Some('-') => Sign::from(chars.next()),
        _ => Sign::Positive,
    };

    while let Some(c) = chars.peek() {
        if c.is_ascii_digit() {
            lexeme.push(*c);
            _ = chars.next();
        } else {
            break;
        }
    }

    if lexeme.is_empty() {
        return Err(miette!(
            "Expected a number after 'e' while parsing numeric literal"
        ));
    }

    Ok(sign.apply_i64(lexeme.parse::<i64>().into_diagnostic()?))
}

/// Parses a number literal that may contain a trailing "n" to indicate a big
/// int.  Falls through to simply returning the primitive that the lexeme and
/// the base parse to.
fn parse_maybe_big_int(
    chars: &mut CodeIter,
    mut lexeme: String,
    base: u32,
    sign: Sign,
) -> Result<NumberLiteralValue> {
    let is_big_int = matches!(chars.peek(), Some('n'));

    match is_big_int {
        true => {
            _ = chars.next();
            if let Sign::Negative = sign {
                lexeme.insert(0, '-');
            }

            let value = num_bigint::BigInt::parse_bytes(lexeme.as_bytes(), base)
                .ok_or(miette!("failed to parse '{}' into BigInt", lexeme))?;
            // TODO: write a "pretty formatter" for big int based on the base,
            // e.g. we want "0xFFn", not "FF"
            lexeme.push('n');
            Ok(NumberLiteralValue::BigInt(BigIntStorage { value, lexeme }))
        }
        false => {
            let value = match base {
                10 => lexeme.parse::<f64>().into_diagnostic()?,
                _ => i64::from_str_radix(&lexeme, base).into_diagnostic()? as f64,
            };

            Ok(NumberLiteralValue::Primitive(sign.apply_f64(value)))
        }
    }
}

fn parse_base_10(chars: &mut CodeIter, sign: Sign) -> Result<NumberLiteralValue> {
    let mut lexeme = String::new();

    'number: while let Some(c) = chars.peek() {
        if is_numeric_separator(*c) {
            _ = chars.next();
            continue 'number;
        }

        if c.is_ascii_digit() || *c == '.' {
            lexeme.push(*c);
            _ = chars.next();
        } else {
            break 'number;
        }
    }

    let exponent = match chars.peek() {
        Some('e') => Some(parse_scientific_exponent(chars)?),
        _ => None,
    };

    match exponent {
        Some(exponent) => {
            Ok((lexeme.parse::<f64>().into_diagnostic()? * 10f64.powi(exponent as i32)).into())
        }
        None => parse_maybe_big_int(chars, lexeme, 10, sign),
    }
}

fn consume_while(iter: &mut CodeIter, predicate: fn(char) -> bool) -> String {
    let mut lexeme = String::new();
    while let Some(c) = iter.peek() {
        if is_numeric_separator(*c) {
            _ = iter.next();
            continue;
        }
        if predicate(*c) {
            lexeme.push(*c);
            _ = iter.next();
        } else {
            break;
        }
    }

    lexeme
}

fn parse_hex_number(chars: &mut CodeIter, sign: Sign) -> Result<NumberLiteralValue> {
    let lexeme = consume_while(chars, |c| c.is_ascii_hexdigit());

    if lexeme.is_empty() {
        return Err(miette!(
            "Expected a valid hexadecimal digit after '0x' while parsing numeric literal"
        ));
    }

    parse_maybe_big_int(chars, lexeme, 16, sign)
}

fn parse_bin_number(chars: &mut CodeIter, sign: Sign) -> Result<NumberLiteralValue> {
    let lexeme = consume_while(chars, |c| c == '0' || c == '1');

    if lexeme.is_empty() {
        return Err(miette!(
            "Expected a valid binary digit after '0b' while parsing numeric literal"
        ));
    }

    parse_maybe_big_int(chars, lexeme, 2, sign)
}

fn parse_oct_number(chars: &mut CodeIter, sign: Sign) -> Result<NumberLiteralValue> {
    let lexeme = consume_while(chars, |c| c.is_oct_digit());

    if lexeme.is_empty() {
        return Err(miette!(
            "Expected a valid octal digit while parsing octal-formatted numeric literal"
        ));
    }

    parse_maybe_big_int(chars, lexeme, 8, sign)
}

/// Attempts to parse a number out of a lexeme that begins with a leading "0".
/// For example, the literal number "0", or differently-based values like
/// hexadecimal or binary.
fn parse_leading_zero_number(chars: &mut CodeIter, sign: Sign) -> Result<NumberLiteralValue> {
    // Consume leading zero:
    _ = chars.next();

    match chars.peek() {
        Some('x') | Some('X') => {
            _ = chars.next();
            parse_hex_number(chars, sign)
        }
        Some('b') | Some('B') => {
            _ = chars.next();
            parse_bin_number(chars, sign)
        }
        Some('o') | Some('O') => {
            _ = chars.next();
            parse_oct_number(chars, sign)
        }
        Some('_') => Err(miette!("Numeric separator can not be used after leading 0")),
        // TODO: support switching on whether legacy octals are allowed:
        Some(c) if c.is_ascii_digit() => parse_oct_number(chars, sign),
        _ => Ok(0.into()),
    }
}

/// Attempts to parse a number out of an iterator of characters.
///
/// Returns:
///
/// * `Ok(Some(NumberLiteral))` - a number literal was successfully parsed out of
/// the iterator.  The iterator has been advanced to the end of the number.
///
/// * `Ok(None)` - the next character of the iterator did not begin a number literal.
///
/// * `Err` - the next character of the iterator began a number literal,
/// but it was malformed or otherwise unable to be parsed.
pub fn try_parse_number(chars: &mut CodeIter) -> Result<Option<NumberLiteralValue>> {
    let sign = match chars.peek() {
        Some('+') | Some('-') => Sign::from(chars.next()),
        _ => Sign::Positive,
    };

    match chars.peek() {
        Some(c) if c.is_ascii_digit() && *c != '0' => parse_base_10(chars, sign).map(Some),
        Some(c) if c.is_ascii_digit() && *c == '0' => {
            parse_leading_zero_number(chars, sign).map(Some)
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::code_iter::IntoCodeIterator;

    use super::*;

    #[test]
    fn test_not_leading_digit_returns_none() {
        let src = "asdf";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(try_parse_number(&mut chars).unwrap(), None);
        assert_eq!(chars.next(), Some('a'));
    }

    #[test]
    fn test_parse_simple_integer() {
        let src = "123A";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 123.into());
        assert_eq!(chars.next().unwrap(), 'A');
    }

    #[test]
    fn test_parse_simple_float() {
        let src = "123.01";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_number(&mut chars).unwrap().unwrap(),
            123.01.into()
        );
    }

    #[test]
    fn test_scientific_notation_integer() {
        let src = "123e4";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 123e4.into());
    }

    #[test]
    fn test_scientific_notation_float() {
        let src = "123.1e2";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_number(&mut chars).unwrap().unwrap(),
            123.1e2.into()
        );
    }

    #[test]
    fn test_base_10_big_int() {
        let src = "123n";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_number(&mut chars).unwrap().unwrap(),
            NumberLiteralValue::BigInt(BigIntStorage {
                value: num_bigint::BigInt::parse_bytes(b"123", 10).unwrap(),
                lexeme: "123n".to_string(),
            })
        );
    }

    #[test]
    fn test_try_float_big_int() {
        let src = "123.3n";
        let mut chars = src.into_code_iterator("script.js".to_string());
        let result = try_parse_number(&mut chars);
        assert_eq!(
            result.unwrap_err().to_string(),
            "failed to parse '123.3' into BigInt"
        );
    }

    #[test]
    fn test_parse_negative_integer() {
        let src = "-123";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_number(&mut chars).unwrap().unwrap(),
            (-123).into()
        );
    }

    #[test]
    fn test_negative_big_int() {
        let src = "-123n";
        let mut chars = src.into_code_iterator("script.js".to_string());

        assert_eq!(
            try_parse_number(&mut chars).unwrap().unwrap(),
            NumberLiteralValue::BigInt(BigIntStorage {
                value: num_bigint::BigInt::parse_bytes(b"-123", 10).unwrap(),
                lexeme: "-123n".to_string(),
            })
        );
    }

    #[test]
    fn test_negative_scientific_notation() {
        let src = "123e-1";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_number(&mut chars).unwrap().unwrap(),
            123e-1.into()
        );
    }

    #[test]
    fn test_zero() {
        let src = "0";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 0.into());
    }

    #[test]
    fn test_hexadecimal_number() {
        let src = "0xFF";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 255.into());
    }

    #[test]
    fn test_negative_hexadecimal_number() {
        let src = "-0xFF";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_number(&mut chars).unwrap().unwrap(),
            (-255).into()
        );
    }

    #[test]
    fn test_hexadecimal_big_int() {
        let src = "0xFFn";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_number(&mut chars).unwrap().unwrap(),
            NumberLiteralValue::BigInt(BigIntStorage {
                value: num_bigint::BigInt::parse_bytes(b"255", 10).unwrap(),
                lexeme: "FFn".to_string(),
            })
        );
    }

    #[test]
    fn test_bin_number() {
        let src = "0b101";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 5.into());
    }

    #[test]
    fn test_strict_octal_number() {
        let src = "0o123";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 83.into());
    }

    #[test]
    fn test_legacy_octal_number() {
        let src = "0123";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 83.into());
    }

    #[test]
    fn test_num_with_underlines() {
        let src = "1_2_3";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 123.into());
    }

    #[test]
    fn test_hex_with_underlines() {
        let src = "0xF_F";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 255.into());
    }

    #[test]
    fn test_hex_with_invalid_numeric_separator() {
        let src = "0_xF_F";
        let mut chars = src.into_code_iterator("script.js".to_string());
        let result = try_parse_number(&mut chars);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Numeric separator can not be used after leading 0"
        );
    }

    #[test]
    fn test_binary_invalid_chars() {
        let src = "0b2";
        let mut chars = src.into_code_iterator("script.js".to_string());
        let result = try_parse_number(&mut chars);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Expected a valid binary digit after '0b' while parsing numeric literal"
        );
    }

    #[test]
    fn test_octal_invalid_chars() {
        let src = "0o8";
        let mut chars = src.into_code_iterator("script.js".to_string());
        let result = try_parse_number(&mut chars);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Expected a valid octal digit while parsing octal-formatted numeric literal"
        );
    }

    #[test]
    fn test_hex_invalid_chars() {
        let src = "0xG";
        let mut chars = src.into_code_iterator("script.js".to_string());
        let result = try_parse_number(&mut chars);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Expected a valid hexadecimal digit after '0x' while parsing numeric literal"
        );
    }
}
