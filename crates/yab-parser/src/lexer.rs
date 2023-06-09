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

impl Ident {
    pub fn new(lexeme: String) -> Self {
        Self { lexeme }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct StringLiteral {
    lexeme: String,
}

impl StringLiteral {
    pub fn new(lexeme: String) -> Self {
        Self { lexeme }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum Token {
    Keyword(Keyword),
    Ident(Ident),
    StringLiteral(StringLiteral),
}

#[derive(Debug, Deserialize, Serialize, PartialEq, HasPrefixLookup)]
pub enum Operator {
    #[token(lexeme = "+")]
    Plus,
    #[token(lexeme = "=")]
    Assignment,
    #[token(lexeme = "==")]
    LooseEquality,
    #[token(lexeme = "===")]
    StrictEquality,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, EnumString)]
pub enum Punctuation {
    #[strum(serialize = ";")]
    Semicolon,
}

pub fn tokenize(source: &str) -> Vec<Token> {
    // probabably does wonky things with non-ascii characters
    let mut chars = source.chars().peekable();
    let mut tokens = Vec::<Token>::new();

    'outer: while let Some(current_char) = chars.next() {
        if current_char.is_whitespace() {
            continue 'outer;
        }

        if matches!(current_char, '\'' | '"') {
            // Discard string delimiter
            _ = chars.next();

            let mut lexeme = String::from(current_char);
            'string: for next_char in chars.by_ref() {
                if matches!(next_char, '\'' | '"') {
                    break 'string;
                }

                lexeme.push(next_char);
            }
            // Discard string delimiter
            _ = chars.next();

            // TODO: handle unexpected EOF
            tokens.push(Token::StringLiteral(StringLiteral::new(lexeme)));
        }

        if current_char.is_alphabetic() {
            let mut lexeme = String::from(current_char);

            'ident: for next_char in chars.by_ref() {
                if next_char.is_alphabetic() {
                    lexeme.push(next_char)
                } else {
                    break 'ident;
                }
            }

            match Keyword::try_from(lexeme.as_str()) {
                Ok(keyword) => tokens.push(Token::Keyword(keyword)),
                Err(_) => tokens.push(Token::Ident(Ident::new(lexeme))),
            }
        }

        // Could the next character _begin_ an operator?
    }

    tokens
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

        Ok(())
    }

    #[test]
    fn it_tokenizes_a_variable_declaration() -> Result<()> {
        let src = "const a = b;";

        assert_eq!(tokenize(src), vec![Token::Keyword("const".try_into()?)]);

        Ok(())
    }
}
