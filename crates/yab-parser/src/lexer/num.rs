#![allow(dead_code)]
use color_eyre::{eyre::eyre, Result};
use nom::AsChar;
use serde::Serialize;
use std::{iter::Peekable, str::Chars};

#[derive(Debug, Serialize)]
pub struct NumberLiteral {
    pub value: NumberLiteralValue,
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
fn parse_scientific_exponent(chars: &mut Peekable<Chars>) -> Result<i64> {
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
        return Err(eyre!(
            "Expected a number after 'e' while parsing numeric literal"
        ));
    }

    Ok(sign.apply_i64(lexeme.parse::<i64>()?))
}

/// Parses a number literal that may contain a trailing "n" to indicate a big
/// int.  Falls through to simply returning the primitive that the lexeme and
/// the base parse to.
fn parse_maybe_big_int(
    chars: &mut Peekable<Chars>,
    mut lexeme: String,
    base: u32,
    sign: Sign,
) -> Result<NumberLiteralValue> {
    let is_big_int = match chars.peek() {
        Some('n') => true,
        _ => false,
    };

    match is_big_int {
        true => {
            _ = chars.next();
            match sign {
                Sign::Negative => lexeme.insert(0, '-'),
                _ => {}
            }
            let value = num_bigint::BigInt::parse_bytes(lexeme.as_bytes(), base)
                .ok_or(eyre!("failed to parse '{}' into BigInt", lexeme))?;
            // TODO: write a "pretty formatter" for big int based on the base,
            // e.g. we want "0xFFn", not "FF"
            lexeme.push('n');
            Ok(NumberLiteralValue::BigInt(BigIntStorage { value, lexeme }))
        }
        false => {
            let value = match base {
                10 => lexeme.parse::<f64>()?,
                _ => i64::from_str_radix(&lexeme, base)? as f64,
            };

            Ok(NumberLiteralValue::Primitive(sign.apply_f64(value)))
        }
    }
}

fn parse_base_10(chars: &mut Peekable<Chars>, sign: Sign) -> Result<NumberLiteralValue> {
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
        Some(exponent) => Ok((lexeme.parse::<f64>()? * 10f64.powi(exponent as i32)).into()),
        None => parse_maybe_big_int(chars, lexeme, 10, sign),
    }
}

fn consume_while(iter: &mut Peekable<Chars>, predicate: fn(char) -> bool) -> String {
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

fn parse_hex_number(chars: &mut Peekable<Chars>, sign: Sign) -> Result<NumberLiteralValue> {
    let lexeme = consume_while(chars, |c| c.is_ascii_hexdigit());
    Ok(parse_maybe_big_int(chars, lexeme, 16, sign)?)
}

fn parse_bin_number(chars: &mut Peekable<Chars>, sign: Sign) -> Result<NumberLiteralValue> {
    let lexeme = consume_while(chars, |c| c == '0' || c == '1');
    Ok(parse_maybe_big_int(chars, lexeme, 2, sign)?)
}

fn parse_oct_number(chars: &mut Peekable<Chars>, sign: Sign) -> Result<NumberLiteralValue> {
    let lexeme = consume_while(chars, |c| c.is_oct_digit());
    Ok(parse_maybe_big_int(chars, lexeme, 8, sign)?)
}

/// Attempts to parse a number out of a lexeme that begins with a leading "0".
/// For example, the literal number "0", or differently-based values like
/// hexadecimal or binary.
fn parse_leading_zero_number(
    chars: &mut Peekable<Chars>,
    sign: Sign,
) -> Result<NumberLiteralValue> {
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
        Some('_') => return Err(eyre!("Numeric separator can not be used after leading 0")),
        // TODO: support switching on whether legacy octals are allowed:
        Some(c) if c.is_ascii_digit() => parse_oct_number(chars, sign),
        _ => Ok(0.into()),
    }
}

/// Attempts to parse a number out of an iterator of characters.
///
/// Returns:
///
/// * Ok(Some(NumberLiteral)) - a number literal was successfully parsed out of
/// the iterator.  The iterator has been advanced to the end of the number.
///
/// * Ok(None) - the next character of the iterator did not begin a number literal.
///
/// * Err(Error) - the next character of the iterator began a number literal,
/// but it was malformed or otherwise unable to be parsed.
pub fn try_parse_number(chars: &mut Peekable<Chars>) -> Result<Option<NumberLiteralValue>> {
    let sign = match chars.peek() {
        Some('+') | Some('-') => Sign::from(chars.next()),
        _ => Sign::Positive,
    };

    match chars.peek() {
        Some(c) if c.is_ascii_digit() && *c != '0' => parse_base_10(chars, sign).map(|v| Some(v)),
        Some(c) if c.is_ascii_digit() && *c == '0' => {
            parse_leading_zero_number(chars, sign).map(|v| Some(v))
        }
        _ => Ok(None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_not_leading_digit_returns_none() {
        let src = "asdf";
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_number(&mut chars).unwrap(), None);
        assert_eq!(chars.next(), Some('a'));
    }

    #[test]
    fn test_parse_simple_integer() {
        let src = "123A";
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 123.into());
        assert_eq!(chars.next().unwrap(), 'A');
    }

    #[test]
    fn test_parse_simple_float() {
        let src = "123.01";
        let mut chars = src.chars().peekable();
        assert_eq!(
            try_parse_number(&mut chars).unwrap().unwrap(),
            123.01.into()
        );
    }

    #[test]
    fn test_scientific_notation_integer() {
        let src = "123e4";
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 123e4.into());
    }

    #[test]
    fn test_scientific_notation_float() {
        let src = "123.1e2";
        let mut chars = src.chars().peekable();
        assert_eq!(
            try_parse_number(&mut chars).unwrap().unwrap(),
            123.1e2.into()
        );
    }

    #[test]
    fn test_base_10_big_int() {
        let src = "123n";
        let mut chars = src.chars().peekable();
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
        let mut chars = src.chars().peekable();
        let result = try_parse_number(&mut chars);
        assert_eq!(
            result.unwrap_err().to_string(),
            "failed to parse '123.3' into BigInt"
        );
    }

    #[test]
    fn test_parse_negative_integer() {
        let src = "-123";
        let mut chars = src.chars().peekable();
        assert_eq!(
            try_parse_number(&mut chars).unwrap().unwrap(),
            (-123).into()
        );
    }

    #[test]
    fn test_negative_big_int() {
        let src = "-123n";
        let mut chars = src.chars().peekable();

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
        let mut chars = src.chars().peekable();
        assert_eq!(
            try_parse_number(&mut chars).unwrap().unwrap(),
            123e-1.into()
        );
    }

    #[test]
    fn test_zero() {
        let src = "0";
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 0.into());
    }

    #[test]
    fn test_hexadecimal_number() {
        let src = "0xFF";
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 255.into());
    }

    #[test]
    fn test_negative_hexadecimal_number() {
        let src = "-0xFF";
        let mut chars = src.chars().peekable();
        assert_eq!(
            try_parse_number(&mut chars).unwrap().unwrap(),
            (-255).into()
        );
    }

    #[test]
    fn test_hexadecimal_big_int() {
        let src = "0xFFn";
        let mut chars = src.chars().peekable();
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
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 5.into());
    }

    #[test]
    fn test_strict_octal_number() {
        let src = "0o123";
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 83.into());
    }

    #[test]
    fn test_legacy_octal_number() {
        let src = "0123";
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 83.into());
    }

    #[test]
    fn test_num_with_underlines() {
        let src = "1_2_3";
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 123.into());
    }

    #[test]
    fn test_hex_with_underlines() {
        let src = "0xF_F";
        let mut chars = src.chars().peekable();
        assert_eq!(try_parse_number(&mut chars).unwrap().unwrap(), 255.into());
    }

    #[test]
    fn test_hex_with_invalid_numeric_separator() {
        let src = "0_xF_F";
        let mut chars = src.chars().peekable();
        let result = try_parse_number(&mut chars);
        assert_eq!(
            result.unwrap_err().to_string(),
            "Numeric separator can not be used after leading 0"
        );
    }
}
