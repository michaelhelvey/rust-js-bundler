use serde::{Deserialize, Serialize};
use strum_macros::EnumString;
use yab_parser_macros::HasPrefixLookup;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum Token {
    Keyword(Keyword),
    Ident(Ident),
    StringLiteral(StringLiteral),
    TemplateLiteralOpen(StringLiteral),
    TemplateLiteralContent(StringLiteral),
    TemplateLiteralClose(StringLiteral),
    ValueLiteral(ValueLiteral),
    RegexLiteral(StringLiteral),
    NumberLiteral(NumberLiteral),
    Operator(Operator),
    Punctuation(Punctuation),
}

/// Trait that can be implemented by operators and punctuators to look up how
/// many members of the enum have lexemes that start with a particular prefix.
/// This allows us to cheaply query in the tokenizer whether a given sequence
/// can be parsed as an operator / puntuator / etc.
pub trait HasPrefixLookup {
    fn fields_starting_with(ident: &str) -> usize;
}

#[derive(Debug, Deserialize, Serialize, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum KeywordType {
    Const,
    Return,
    Function,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Keyword {
    keyword_type: KeywordType,
}

impl Keyword {
    pub fn new(keyword_type: KeywordType) -> Self {
        Self { keyword_type }
    }
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
pub struct ValueLiteral {
    value_type: ValueLiteralType,
}

impl ValueLiteral {
    pub fn new(value_type: ValueLiteralType) -> Self {
        Self { value_type }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct NumberLiteral {
    // FIXME: these should not be strings long-term
    value: String,
}

impl NumberLiteral {
    pub fn new(value: String) -> Self {
        Self { value }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum ValueLiteralType {
    True,
    False,
    Null,
}

#[derive(Debug, Deserialize, Serialize, PartialEq, HasPrefixLookup, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum OperatorType {
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

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Operator {
    operator_type: OperatorType,
}

impl Operator {
    pub fn new(operator_type: OperatorType) -> Self {
        Self { operator_type }
    }
}

#[derive(Debug, Deserialize, Serialize, PartialEq, EnumString, HasPrefixLookup)]
pub enum PunctuationType {
    #[token(lexeme = ";")]
    #[strum(serialize = ";")]
    Semicolon,
    #[token(lexeme = "(")]
    #[strum(serialize = "(")]
    OpenParen,
    #[token(lexeme = ")")]
    #[strum(serialize = ")")]
    CloseParen,
    #[token(lexeme = "{")]
    #[strum(serialize = "{")]
    OpenBrace,
    #[token(lexeme = "}")]
    #[strum(serialize = "}")]
    CloseBrace,
    #[token(lexeme = ".")]
    #[strum(serialize = ".")]
    Dot,
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Punctuation {
    punct_type: PunctuationType,
}

impl Punctuation {
    pub fn new(punct_type: PunctuationType) -> Self {
        Self { punct_type }
    }
}
