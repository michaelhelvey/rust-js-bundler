use color_eyre::{eyre::eyre, Result};
use serde::{Deserialize, Serialize};
use strum_macros::EnumString;
use yab_parser_macros::HasPrefixLookup;

/// Trait that can be implemented by operators and punctuators to look up how
/// many members of the enum have lexemes that start with a particular prefix.
/// This allows us to cheaply query in the tokenizer whether a given sequence
/// can be parsed as an operator / puntuator / etc.
trait HasPrefixLookup {
    fn fields_starting_with(ident: &str) -> usize;
}

#[derive(Debug, Deserialize, Serialize, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Keyword {
    Const,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Ident {
    lexeme: String,
}

impl From<String> for Ident {
    fn from(value: String) -> Self {
        Self { lexeme: value }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct StringLiteral {
    lexeme: String,
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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum Token {
    Keyword(Keyword),
    Ident(Ident),
    StringLiteral(StringLiteral),
    Operator(Operator),
    Punctuation(Punctuation),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, HasPrefixLookup, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum Operator {
    #[token(lexeme = "+")]
    #[strum(serialize = "+")]
    Plus,
    #[token(lexeme = "=")]
    #[strum(serialize = "=")]
    Assignment,
    #[token(lexeme = "==")]
    #[strum(serialize = "==")]
    LooseEquality,
    #[token(lexeme = "===")]
    #[strum(serialize = "===")]
    StrictEquality,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, EnumString, HasPrefixLookup)]
pub enum Punctuation {
    #[token(lexeme = ";")]
    #[strum(serialize = ";")]
    Semicolon,
}

macro_rules! tokenize_prefix {
    ($token_type:ident, $current_char:ident, $chars:ident, $tokens:ident, $cont_label:lifetime) => {
        // There doesn't seem to be a way to get a slice out of an iterator, so
        // for now we will just allocate :/
        let mut prefix_lexeme = String::from($current_char);
        let mut prefix_matches = $token_type::fields_starting_with(&prefix_lexeme);

        if prefix_matches > 0 {
            'prefix: while let Some(next_char) = $chars.peek() {
                prefix_lexeme.push(*next_char);
                prefix_matches = $token_type::fields_starting_with(&prefix_lexeme);

                if prefix_matches == 0 {
                    prefix_lexeme = prefix_lexeme[0..prefix_lexeme.len() - 1].to_string();
                    break 'prefix;
                } else {
                    _ = $chars.next()
                }
            }

            let prefix_ref: &str = prefix_lexeme.as_ref();
            let prefix: $token_type = prefix_ref.try_into().expect(&format!(
                "Internal tokenizer error: could not parse {} into $token_type",
                prefix_ref
            ));

            $tokens.push(Token::$token_type(prefix));
            continue $cont_label;
        }
    };
}

fn is_line_separator(c: char) -> bool {
    matches!(c, '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}')
}

pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut chars = source.chars().peekable();
    let mut tokens = Vec::<Token>::new();

    'outer: while let Some(current_char) = chars.next() {
        if current_char.is_whitespace() {
            continue 'outer;
        }

        if matches!(current_char, '\'' | '"') {
            let unexpected_eof_msg = "Unexpected EOF while parsing string";
            let next_char = chars.next().ok_or(eyre!(unexpected_eof_msg))?;

            let mut lexeme = String::from(next_char);

            let mut reached_str_end = false;
            'string: while let Some(next_char) = chars.next() {
                if matches!(next_char, '\'' | '"') {
                    reached_str_end = true;
                    break 'string;
                }

                if is_line_separator(next_char) {
                    return Err(eyre!("Unexpected line terminator while parsing string"));
                }

                if next_char == '\\' {
                    // see: https://tc39.es/ecma262/#prod-EscapeSequence
                    // see: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#escape_sequences
                    let escaped_char = match chars.next() {
                        Some('0') => Ok(Some('\u{0000}')), // NULL
                        Some('\'') => Ok(Some('\'')),
                        Some('"') => Ok(Some('"')),
                        Some('\\') => Ok(Some('\\')),
                        Some('n') => Ok(Some('\n')),       // LINEFEED
                        Some('r') => Ok(Some('\r')),       // CARRIAGE RETURN
                        Some('v') => Ok(Some('\u{000B}')), // LINE TABULATION
                        Some('t') => Ok(Some('\t')),       // TAB
                        Some('b') => Ok(Some('\u{0008}')), // BACKSPACE
                        Some('f') => Ok(Some('\u{000C}')), // FORM FEED
                        Some('u') => {
                            let mut char_seq = String::new();
                            let mut i = 0;
                            while i < 4 {
                                char_seq.push(chars.next().ok_or(eyre!(
                                    "Unexpected EOF while parsing unicode escape sequence"
                                ))?);
                                i += 1;
                            }

                            let unicode_hex_value = u32::from_str_radix(char_seq.as_ref(), 16)?;
                            let unicode_char = char::from_u32(unicode_hex_value);
                            Ok(unicode_char)
                        }
                        Some('x') => {
                            todo!("hex codes");
                        }
                        Some('c') => {
                            todo!("control codes");
                        }
                        // Escaping a line terminator in a source file should
                        // result in an empty string:
                        // see: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Lexical_grammar#line_terminators
                        Some('\u{000A}') => Ok(None),
                        Some('\u{000D}') => Ok(None),
                        Some('\u{2028}') => Ok(None),
                        Some('\u{2029}') => Ok(None),
                        Some(c) => Ok(Some(c)), // NonEscapeCharacter
                        None => Err(eyre!("Unexpected EOF while parsing escape sequence")),
                    };

                    if let Some(c) = escaped_char? {
                        lexeme.push(c);
                    }

                    continue 'string;
                }

                lexeme.push(next_char);
            }

            if chars.peek().is_none() && !reached_str_end {
                return Err(eyre!(unexpected_eof_msg));
            }

            // Discard trailing string delimiter
            _ = chars.next();

            tokens.push(Token::StringLiteral(lexeme.into()));
            continue 'outer;
        }

        if current_char.is_alphabetic() {
            let mut lexeme = String::from(current_char);

            'ident: while let Some(next_char) = chars.peek() {
                if next_char.is_alphabetic() {
                    lexeme.push(*next_char);
                    _ = chars.next();
                } else {
                    break 'ident;
                }
            }

            match Keyword::try_from(lexeme.as_str()) {
                Ok(keyword) => tokens.push(Token::Keyword(keyword)),
                Err(_) => tokens.push(Token::Ident(Ident::from(lexeme))),
            }

            continue 'outer;
        }

        tokenize_prefix!(Operator, current_char, chars, tokens, 'outer);
        tokenize_prefix!(Punctuation, current_char, chars, tokens, 'outer);
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;

    #[test]
    fn test_macro_prefix_lookup() -> Result<()> {
        assert_eq!(Operator::fields_starting_with("="), 3);
        assert_eq!(Operator::fields_starting_with("=="), 2);
        assert_eq!(Operator::fields_starting_with("==="), 1);
        assert_eq!(Operator::fields_starting_with("~!~~"), 0);

        Ok(())
    }

    #[test]
    fn operator_tokenization() -> Result<()> {
        assert_eq!(tokenize("=")?, vec![Token::Operator(Operator::Assignment)]);
        assert_eq!(
            tokenize("==")?,
            vec![Token::Operator(Operator::LooseEquality)]
        );
        assert_eq!(
            tokenize("===")?,
            vec![Token::Operator(Operator::StrictEquality)]
        );
        assert_eq!(tokenize("+")?, vec![Token::Operator(Operator::Plus)]);
        Ok(())
    }

    #[test]
    fn string_literal_simple() -> Result<()> {
        let src = r#""hello""#;
        assert_eq!(tokenize(src)?, vec![Token::StringLiteral("hello".into())]);

        Ok(())
    }

    #[test]
    fn string_literal_unexpected_eof() {
        let invalid_src_1 = r#"""#;
        let result = tokenize(invalid_src_1);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unexpected EOF while parsing string"
        );

        let invalid_src_2 = r#""hello"#;
        let result = tokenize(invalid_src_2);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unexpected EOF while parsing string"
        );
    }

    #[test]
    fn string_literal_escape_sequences() -> Result<()> {
        // "basic" escape sequences like \n, \t, etc:
        let src = r#""h\ello\n""#;
        let result = tokenize(src)?;
        assert_eq!(result, vec![Token::StringLiteral("hello\n".into())]);

        // escaping new line
        let src = r#""hello \
there""#;
        let result = tokenize(src)?;
        assert_eq!(result, vec![Token::StringLiteral("hello there".into())]);

        // escaping unicode sequences
        let src = r#""hello\u0041""#;
        let result = tokenize(src)?;
        assert_eq!(result, vec![Token::StringLiteral("helloA".into())]);

        Ok(())
    }

    #[test]
    fn string_literal_unexpected_lt() {
        let src = r#"
"hello
there"
        "#;
        let result = tokenize(src);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unexpected line terminator while parsing string"
        );
    }

    #[test]
    fn it_tokenizes_a_variable_declaration() -> Result<()> {
        let src = "const a = b;";

        assert_eq!(
            tokenize(src)?,
            vec![
                Token::Keyword("const".try_into()?),
                Token::Ident(Ident::from("a".to_string())),
                Token::Operator(Operator::Assignment),
                Token::Ident(Ident::from("b".to_string())),
                Token::Punctuation(Punctuation::Semicolon),
            ]
        );

        Ok(())
    }
}
