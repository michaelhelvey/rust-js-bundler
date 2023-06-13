use color_eyre::{eyre::eyre, Result};
use serde::Serialize;

use self::{
    comment::Comment,
    ident::{IdentParseResult, Identifier, Keyword},
    operator::Operator,
    punctuation::Punctuation,
};

mod comment;
mod escape_chars;
mod ident;
mod num;
mod operator;
mod punctuation;
mod regex;
mod string;
mod template;
mod utils;

// todo: null & boolean literals
// #[derive(Debug, Deserialize, Serialize, PartialEq)]
// pub struct ValueLiteral {
//     value_type: ValueLiteralType,
// }

// impl ValueLiteral {
//     pub fn new(value_type: ValueLiteralType) -> Self {
//         Self { value_type }
//     }
// }

// #[derive(Debug, Deserialize, Serialize, PartialEq, EnumString)]
// #[strum(serialize_all = "snake_case")]
// pub enum ValueLiteralType {
//     True,
//     False,
//     Null,
// }

#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum Token {
    Keyword(Keyword),
    Ident(Identifier),
    Operator(Operator),
    Punctuation(Punctuation),
    Comment(Comment),
}

pub fn tokenize(src: &str) -> Result<Vec<Token>> {
    let mut chars = src.chars().peekable();
    let mut tokens = Vec::<Token>::new();

    'outer: loop {
        if let None = chars.peek() {
            break;
        }

        if let Some(next_char) = chars.peek() {
            if next_char.is_whitespace() {
                chars.next();
                continue 'outer;
            }
        }

        if let Some(comment) = comment::try_parse_comment(&mut chars) {
            tokens.push(Token::Comment(comment));
            continue 'outer;
        }

        if let Some(parse_result) = ident::try_parse_identifier(&mut chars)? {
            match parse_result {
                IdentParseResult::Identifier(ident) => {
                    tokens.push(Token::Ident(ident));
                }
                IdentParseResult::Keyword(keyword) => {
                    tokens.push(Token::Keyword(keyword));
                }
            }

            continue 'outer;
        }

        if let Some(punctuation) = punctuation::try_parse_punctuation(&mut chars) {
            tokens.push(Token::Punctuation(punctuation));
            continue 'outer;
        }

        if let Some(operator) = operator::try_parse_operator(&mut chars) {
            tokens.push(Token::Operator(operator));
            continue 'outer;
        }

        return Err(eyre!(
            "Unexpected character: '{}'",
            chars.peek().unwrap_or(&'?')
        ));
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use crate::lexer::{
        comment::CommentType, operator::OperatorType, punctuation::PunctuationType,
    };

    use super::*;

    #[test]
    fn test_file_tokenization() -> Result<()> {
        let src = r#"
// This is a a comment
const a = b;
"#;

        assert_eq!(
            tokenize(src).unwrap(),
            vec![
                Token::Comment(Comment::new(CommentType::Line(
                    " This is a a comment".to_string()
                ))),
                Token::Keyword(Keyword::new("const".try_into()?)),
                Token::Ident("a".into()),
                Token::Operator(Operator::new(OperatorType::Assignment)),
                Token::Ident("b".into()),
                Token::Punctuation(Punctuation::new(PunctuationType::Semicolon)),
            ]
        );

        Ok(())
    }

    //     #[test]
    //     fn sanity_check_large_expression() -> Result<()> {
    //         let src = r#"
    // const a = `my template: ${b}`;

    // function foo() {
    //     return /hello/gm.test("\u0041BC");
    // }
    // "#;

    //         assert_eq!(
    //             tokenize(src)?,
    //             vec![
    //                 Token::Keyword(Keyword::new("const".try_into()?)),
    //                 Token::Ident(Ident::from("a".to_string())),
    //                 Token::Operator(Operator::new(OperatorType::Assignment)),
    //                 Token::TemplateLiteralOpen(StringLiteral::from("`my template: ${")),
    //                 Token::Ident(Ident::from("b".to_string())),
    //                 Token::TemplateLiteralClose(StringLiteral::from("}`")),
    //                 Token::Punctuation(Punctuation::new(PunctuationType::Semicolon)),
    //                 Token::Keyword(Keyword::new("function".try_into()?)),
    //                 Token::Ident(Ident::from("foo".to_string())),
    //                 Token::Punctuation(Punctuation::new(PunctuationType::OpenParen)),
    //                 Token::Punctuation(Punctuation::new(PunctuationType::CloseParen)),
    //                 Token::Punctuation(Punctuation::new(PunctuationType::OpenBrace)),
    //                 Token::Keyword(Keyword::new("return".try_into()?)),
    //                 Token::RegexLiteral(StringLiteral::from("/hello/gm".to_string())),
    //                 Token::Punctuation(Punctuation::new(PunctuationType::Dot)),
    //                 Token::Ident(Ident::from("test".to_string())),
    //                 Token::Punctuation(Punctuation::new(PunctuationType::OpenParen)),
    //                 Token::StringLiteral(StringLiteral::from("ABC".to_string())),
    //                 Token::Punctuation(Punctuation::new(PunctuationType::CloseParen)),
    //                 Token::Punctuation(Punctuation::new(PunctuationType::Semicolon)),
    //                 Token::Punctuation(Punctuation::new(PunctuationType::CloseBrace)),
    //             ]
    //         );

    //         Ok(())
    //     }
}
