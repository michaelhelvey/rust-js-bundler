use miette::Result;
use serde::Serialize;
use strum_macros::EnumString;

use super::{
    code_iter::{current_span_error, CodeIter, Span},
    escape_chars::try_parse_escape,
    operator::{Operator, OperatorType},
};

#[derive(Debug, PartialEq)]
pub enum IdentParseResult {
    Identifier(Identifier),
    Keyword(Keyword),
    ValueLiteral(ValueLiteral),
    Operator(Operator),
}

#[derive(Debug, Serialize, PartialEq)]
pub struct ValueLiteral {
    kind: ValueLiteralType,
}

impl ValueLiteral {
    pub fn new(kind: ValueLiteralType) -> Self {
        Self { kind }
    }
}

#[derive(Debug, Serialize, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ValueLiteralType {
    True,
    False,
    Null,
}

#[derive(Debug, Serialize, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum KeywordType {
    Async,
    Const,
    Function,
    Import,
    Export,
    New,
    Return,
    Super,
    This,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Keyword {
    kind: KeywordType,
}

impl Keyword {
    pub fn new(kind: KeywordType) -> Self {
        Self { kind }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct Identifier {
    lexeme: String,
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Self { lexeme: value }
    }
}

impl From<&str> for Identifier {
    fn from(value: &str) -> Self {
        Self {
            lexeme: value.to_string(),
        }
    }
}

/// Attempts to parse a valid Javascript identifier from an iterator.  See:
/// https://tc39.es/ecma262/#prod-IdentifierName
///
/// Returns:
///
/// * `Ok(Some(Identifier))` if the iterator starts with a valid identifier.
/// Stops parsing the identifier as soon as an invalid identifier character is
/// reached.
///
/// * `Ok(None)` if the iterator does not begin with a valid identifier character.
///
/// * `Err` if an invalid escape sequence is encountered.
pub fn try_parse_identifier(chars: &mut CodeIter) -> Result<Option<IdentParseResult>> {
    let start_pos = chars.current_position();
    let mut lexeme = String::new();

    let mut at_start = true;
    'ident: while let Some(next_char) = chars.peek() {
        let mut requires_advancing = true;

        let token_pred = |c: char| {
            if at_start {
                c.is_alphabetic() || c == '_' || c == '$'
            } else {
                c.is_alphanumeric() || c == '_' || c == '$'
            }
        };

        let next_char = match next_char {
            '\\' => {
                _ = chars.next();
                requires_advancing = false;
                let escaped_char = try_parse_escape(chars)?;

                match escaped_char {
                    Some(c) if !token_pred(c) => {
                        return Err(current_span_error!(
                            chars,
                            start_pos,
                            "Invalid escape sequence in identifier: \\u{:04X}",
                            c as u32
                        ));
                    }
                    Some(c) => c,
                    _ => continue 'ident,
                }
            }
            _ => *next_char,
        };

        if token_pred(next_char) {
            lexeme.push(next_char);
            if requires_advancing {
                _ = chars.next();
            }
        } else {
            break;
        }

        at_start = false;
    }

    if lexeme.is_empty() {
        return Ok(None);
    }

    if let Ok(keyword_type) = KeywordType::try_from(lexeme.as_str()) {
        return Ok(Some(IdentParseResult::Keyword(Keyword::new(keyword_type))));
    }

    if let Ok(operator_type) = OperatorType::try_from(lexeme.as_str()) {
        return Ok(Some(IdentParseResult::Operator(Operator::new(
            operator_type,
        ))));
    }

    if let Ok(value_type) = ValueLiteralType::try_from(lexeme.as_str()) {
        return Ok(Some(IdentParseResult::ValueLiteral(ValueLiteral::new(
            value_type,
        ))));
    }

    Ok(Some(IdentParseResult::Identifier(lexeme.into())))
}

#[cfg(test)]
mod tests {
    use crate::lexer::code_iter::IntoCodeIterator;
    use miette::miette;

    use super::*;

    #[test]
    fn test_parse_simple_identifier() {
        let src = "hello";
        let mut chars = src.into_code_iterator("script.js".to_string());

        assert_eq!(
            try_parse_identifier(&mut chars).unwrap().unwrap(),
            IdentParseResult::Identifier(Identifier::from("hello"))
        );
    }

    #[test]
    fn test_parse_beginning_underscore_identifier() {
        let src = "_hello";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_identifier(&mut chars).unwrap().unwrap(),
            IdentParseResult::Identifier(Identifier::from("_hello"))
        );
    }

    #[test]
    fn test_parse_numeric_identifier() {
        let src = "_hello123";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_identifier(&mut chars).unwrap().unwrap(),
            IdentParseResult::Identifier(Identifier::from("_hello123"))
        );
    }

    #[test]
    fn test_parse_unicode_start_id() {
        let src = r#"\u0041BC"#;
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_identifier(&mut chars).unwrap().unwrap(),
            IdentParseResult::Identifier(Identifier::from("ABC"))
        );
    }

    #[test]
    fn test_parse_unicode_mid() {
        let src = r#"A\u0042C"#;
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_identifier(&mut chars).unwrap().unwrap(),
            IdentParseResult::Identifier(Identifier::from("ABC"))
        );
    }

    #[test]
    fn test_invalid_identifer() {
        let src = r#"AB\u0043\n"#;
        let mut chars = src.into_code_iterator("script.js".to_string());
        let result = try_parse_identifier(&mut chars);

        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid escape sequence in identifier: \\u000A"));
    }

    #[test]
    fn test_identifier_parser_does_not_eat_trailing_chars() {
        let src = r#"AB\u0043 "#;
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_identifier(&mut chars).unwrap().unwrap(),
            IdentParseResult::Identifier(Identifier::from("ABC"))
        );
        assert_eq!(chars.next().unwrap(), ' ');
    }

    #[test]
    fn test_keyword() -> Result<()> {
        let src = "const a";
        let mut chars = src.into_code_iterator("script.js".to_string());
        assert_eq!(
            try_parse_identifier(&mut chars).unwrap().unwrap(),
            IdentParseResult::Keyword(Keyword::new(
                "const"
                    .try_into()
                    .map_err(|e| miette!("Could not parse keyword: {}", e))?
            ))
        );
        assert_eq!(chars.next().unwrap(), ' ');

        Ok(())
    }

    #[test]
    fn test_parse_value_types() {
        let value_literals = vec![
            (ValueLiteralType::Null, "null"),
            (ValueLiteralType::True, "true"),
            (ValueLiteralType::False, "false"),
        ];

        for (value_type, src) in value_literals {
            let mut chars = src.into_code_iterator("script.js".to_string());
            assert_eq!(
                try_parse_identifier(&mut chars).unwrap().unwrap(),
                IdentParseResult::ValueLiteral(ValueLiteral::new(value_type))
            );
        }
    }
}
