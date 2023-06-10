use color_eyre::{eyre::eyre, Result};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete::{digit0, digit1, hex_digit1},
    combinator::opt,
    error::Error,
    sequence::tuple,
    IResult,
};
use num_bigint::BigInt;
use num_traits::Num;

/// Parses the optional prefix for a number (e.g. 0x, 0b, 0o), returning the
/// numeric part after the prefix, along with the base that the prefix indicates.
fn parse_prefix(lexeme: &str) -> IResult<&str, u32> {
    let (remaining, prefix) = alt((
        tag("0x"),
        tag("0X"),
        tag("0b"),
        tag("0B"),
        tag("0o"),
        tag("0O"),
    ))(lexeme)?;

    let base = match prefix {
        "0x" | "0X" => 16,
        "0b" | "0B" => 2,
        "0o" | "0O" => 8,
        _ => panic!("unreachable"),
    };

    Ok((remaining, base))
}

struct NumberParseResult {
    is_float: bool,
    lexeme: String,
}

enum NumberSuffixResult {
    Exponent(i32),
    BigInt,
}

/// Parses the optional suffix for a number (e.g. n, i, f, e3, E3, etc.)
fn parse_suffix(lexeme: &str) -> Result<Option<NumberSuffixResult>> {
    // This is why they make fun of you, Rust :/
    let e_parser = alt((tag::<&str, &str, Error<&str>>("e"), tag("E")));
    let sign_parser = alt((tag::<&str, &str, Error<&str>>("+"), tag("-")));
    let n_parser = tag::<&str, &str, Error<&str>>("n");

    // You can _either_ have an exponent, _or_ you can have an N
    let (remaining, exponent) = match tuple((e_parser, opt(sign_parser), digit1))(lexeme) {
        Ok((remaining, (_, sign, digits))) => {
            // I think the only thing that this eror could possibly be is overflow.
            let mut exponent_value: i32 = digits.parse::<i32>()?;
            exponent_value = match sign {
                Some("-") => -(exponent_value),
                _ => exponent_value,
            };

            (remaining, Some(exponent_value))
        }
        Err(_) => (lexeme, None),
    };

    match exponent {
        Some(exponent) => {
            // If we have an exponent, then we can't have an "n" suffix.
            if n_parser(remaining).is_ok() {
                return Err(eyre!("Unexpected 'n' after exponent in base-10 number"));
            }

            return Ok(Some(NumberSuffixResult::Exponent(exponent)));
        }
        None => match n_parser(remaining) {
            Ok((_, _)) => return Ok(Some(NumberSuffixResult::BigInt)),
            Err(_) => return Ok(None),
        },
    }
}

/// Given the numeric part of a number, e.g. the "FFe3" in "0xFFe3", parses the
/// number into the "FF" and "e3" parts.
fn parse_number_part(lexeme: &str, base: u32) -> IResult<&str, NumberParseResult> {
    let pre_number_parser = take_while(|c: char| {
        let is_valid_number = match base {
            10 => c.is_ascii_digit(),
            16 => c.is_ascii_hexdigit(),
            8 => matches!(c, '0'..='7'),
            2 => c == '0' || c == '1',
            // Panicking probably isn't ideal here but it works for now.  If we
            // make it here we've done something very very wrong in our base
            // parsing code.
            _ => panic!("unreachable"),
        };
        return is_valid_number || c == '_';
    });
    let after_number_parser = alt((digit0, tag("_")));

    let (suffix, (before_decimal, decimal, after_decimal)) =
        tuple((pre_number_parser, opt(tag(".")), after_number_parser))(lexeme)?;

    let is_float = decimal.is_some();

    let mut lexeme = String::from(before_decimal);
    lexeme.push_str(decimal.unwrap_or(""));
    lexeme.push_str(after_decimal);

    Ok((
        suffix,
        NumberParseResult {
            is_float,
            lexeme: lexeme.replace("_", ""),
        },
    ))
}

#[derive(Debug, PartialEq)]
pub enum NumberLiteral {
    Primitive(f64),
    BigInteger(num_bigint::BigInt),
}

impl From<f64> for NumberLiteral {
    fn from(value: f64) -> Self {
        Self::Primitive(value)
    }
}

impl From<i64> for NumberLiteral {
    fn from(value: i64) -> Self {
        Self::Primitive(value as f64)
    }
}

impl From<BigInt> for NumberLiteral {
    fn from(value: BigInt) -> Self {
        Self::BigInteger(value)
    }
}

pub fn parse_number(lexeme: &str, allow_sloppy_octal: bool) -> Result<NumberLiteral> {
    // Test for invalid prefixes.  This is ideally done by the lexer itself
    // before this is ever called.
    match alt((tag::<&str, &str, Error<&str>>("-"), tag("+"), hex_digit1))(lexeme) {
        Ok(_) => (),
        Err(_) => return Err(eyre!("Invalid first character in number: {:?}", lexeme)),
    };

    // Parse an optional sig before the number.
    let sign_parser = alt((tag::<&str, &str, Error<&str>>("-"), tag("+")));
    let (without_sign, sign) = match opt(sign_parser)(lexeme) {
        Ok((remaining, sign)) => (remaining, sign),
        Err(_) => (lexeme, None),
    };

    // Parse the base prefix, if any, e.g. 0x, or 0b.
    let (mut remaining, mut base) = match parse_prefix(without_sign) {
        Ok((after_prefix, base)) => (after_prefix, base),
        Err(_) => (without_sign, 10),
    };

    // In non-strict mode, allow sloppy octal parsing with trailing 0s, e.g. 015.
    (remaining, base) = match tuple((tag("0"), digit1::<&str, nom::error::Error<&str>>))(remaining)
    {
        Ok(_) => match allow_sloppy_octal {
            true => Ok((remaining, 8)),
            false => Err(eyre!("Octal literals are not allowed in strict mode")),
        },
        Err(_) => Ok((remaining, base)),
    }?;

    // parse the remaining number into the number part (including possible
    // decimal point), and then the suffix ([eE][+-]?[0-9]+n?)
    let (suffix, number_part) = match parse_number_part(remaining, base) {
        Ok((suffix, number_part)) => (suffix, number_part),
        Err(e) => return Err(eyre!("Failed to parse number: {:?}", e)),
    };

    if base != 10 && number_part.is_float {
        return Err(eyre!("Unexpected decimal while parsing non-base 10 number"));
    }

    eprintln!(
        "number_part: {}, suffix: {}, base: {}, is_float: {}",
        number_part.lexeme, suffix, base, number_part.is_float,
    );

    let suffix_result = parse_suffix(suffix)?;

    if let Some(NumberSuffixResult::BigInt) = suffix_result {
        let lexeme = sign.unwrap_or("+").to_string() + &number_part.lexeme;
        let value = BigInt::from_str_radix(lexeme.as_ref(), base)?;

        return Ok(value.into());
    }

    let mut value = match number_part.is_float {
        true => number_part.lexeme.parse::<f64>()?,
        false => {
            let value = i64::from_str_radix(number_part.lexeme.as_ref(), base)?;
            value as f64
        }
    };

    value = match sign {
        Some("-") => -value,
        _ => value,
    };

    if let Some(NumberSuffixResult::Exponent(exp)) = suffix_result {
        value = value * 10f64.powi(exp);
    }

    Ok(value.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_integer() {
        assert_eq!(parse_number("123".into(), true).unwrap(), 123.0.into());
    }

    #[test]
    fn test_parse_integer_with_separators() {
        assert_eq!(parse_number("12_3".into(), true).unwrap(), 123.0.into());
    }

    #[test]
    fn test_parse_simple_float() {
        assert_eq!(parse_number("123.12".into(), true).unwrap(), 123.12.into());
    }

    #[test]
    fn test_parse_binary() {
        assert_eq!(parse_number("0b11".into(), true).unwrap(), 3.0.into());
    }

    #[test]
    fn test_parse_octal_sloppy() {
        assert_eq!(parse_number("015".into(), true).unwrap(), 13.into());
    }

    #[test]
    fn test_parse_octal_strict() {
        assert_eq!(parse_number("0o15".into(), true).unwrap(), 13.into());
    }

    #[test]
    fn test_parse_hex() {
        assert_eq!(parse_number("0xFF".into(), true).unwrap(), 255.into());
    }

    #[test]
    fn test_parse_negative() {
        assert_eq!(parse_number("-0xFF".into(), true).unwrap(), (-255.0).into());
    }

    #[test]
    fn test_parse_positive() {
        assert_eq!(parse_number("+10".into(), true).unwrap(), 10.into());
    }

    #[test]
    fn test_parse_garbage_prefix() {
        let result = parse_number("~10".into(), true);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Invalid first character in number: \"~10\"",
        );
    }

    #[test]
    fn test_strict_octal_mode_errors() {
        let result = parse_number("015".into(), false);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Octal literals are not allowed in strict mode",
        );
    }

    #[test]
    fn test_decimals_not_allowed_in_non_base_10_numbers() {
        let result = parse_number("0xFF.1".into(), true);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unexpected decimal while parsing non-base 10 number",
        );
    }

    #[test]
    fn test_integer_raised_to_powers() {
        assert_eq!(parse_number("10e2".into(), true).unwrap(), 1000.into());
    }

    #[test]
    fn test_decimal_raised_to_powers() {
        assert_eq!(parse_number("10.1e2".into(), true).unwrap(), 1010.into());
    }

    #[test]
    fn test_negative_decimal_raised_to_negative_power() {
        assert_eq!(
            parse_number("-10.1e-4".into(), true).unwrap(),
            (-0.00101f64).into()
        );
    }

    #[test]
    fn test_base_10_big_int() {
        assert_eq!(
            parse_number("1234567890123456789012345678901234567890n".into(), true).unwrap(),
            BigInt::parse_bytes(b"1234567890123456789012345678901234567890", 10)
                .unwrap()
                .into(),
        );
    }

    #[test]
    fn test_base_16_big_int() {
        assert_eq!(
            parse_number("0x1234567890abcdefABCDEFn".into(), true).unwrap(),
            BigInt::parse_bytes(b"1234567890abcdefABCDEF", 16)
                .unwrap()
                .into(),
        );
    }
}
