use miette::{miette, Result};
use serde::Serialize;

use self::{
    code_iter::IntoCodeIterator,
    comment::Comment,
    ident::{IdentParseResult, Identifier, Keyword, ValueLiteral},
    num::NumberLiteral,
    operator::Operator,
    punctuation::Punctuation,
    regex::RegexLiteral,
    string::StringLiteral,
    template::{TemplateLiteralExprClose, TemplateLiteralExprOpen, TemplateLiteralString},
};

mod code_iter;
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

#[derive(Debug, Serialize, PartialEq)]
#[serde(tag = "type")]
pub enum Token {
    Keyword(Keyword),
    Ident(Identifier),
    ValueLiteral(ValueLiteral),
    Operator(Operator),
    Punctuation(Punctuation),
    Comment(Comment),
    NumericLiteral(NumberLiteral),
    StringLiteral(StringLiteral),
    TemplateLiteralString(TemplateLiteralString),
    TemplateLiteralExprOpen(TemplateLiteralExprOpen),
    TemplateLiteralExprClose(TemplateLiteralExprClose),
    RegexLiteral(RegexLiteral),
}

pub fn tokenize(src: &str) -> Result<Vec<Token>> {
    let mut chars = src.into_code_iterator("script.js".to_string());
    let mut tokens = Vec::<Token>::new();
    let mut template_depth = 0;

    'outer: loop {
        if chars.peek().is_none() {
            break;
        }

        if tokens.is_empty() {
            if let Some(comment) = comment::try_parse_hashbang_comment(&mut chars) {
                tokens.push(Token::Comment(comment));
                continue 'outer;
            }
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

        if let Some((template_content, template_expr_open)) =
            template::try_parse_template_literal_start(&mut chars)?
        {
            template_depth += 1;
            tokens.push(Token::TemplateLiteralString(template_content));

            if let Some(template_expr_open) = template_expr_open {
                tokens.push(Token::TemplateLiteralExprOpen(template_expr_open));
            }

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
                IdentParseResult::ValueLiteral(value_literal) => {
                    tokens.push(Token::ValueLiteral(value_literal));
                }
            }

            continue 'outer;
        }

        if template_depth > 0 {
            if let Some((expr_close, template_content, expr_open)) =
                template::try_parse_template_literal_expr_end(&mut chars)?
            {
                template_depth -= 1;
                tokens.push(Token::TemplateLiteralExprClose(expr_close));
                tokens.push(Token::TemplateLiteralString(template_content));

                if let Some(expr_open) = expr_open {
                    template_depth += 1;
                    tokens.push(Token::TemplateLiteralExprOpen(expr_open));
                }

                continue 'outer;
            }
        }

        if let Some(regexp) = regex::try_parse_regex_literal(&mut chars)? {
            tokens.push(Token::RegexLiteral(regexp));
            continue 'outer;
        }

        if let Some(string_literal) = string::try_parse_string(&mut chars)? {
            tokens.push(Token::StringLiteral(string_literal));
            continue 'outer;
        }

        if let Some(number_value) = num::try_parse_number(&mut chars)? {
            tokens.push(Token::NumericLiteral(NumberLiteral::new(number_value)));
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

        return Err(miette!(
            "Unexpected character: '{}' (last token parsed: {:?})",
            chars.peek().unwrap_or(&'?'),
            tokens.last()
        ));
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use miette::IntoDiagnostic;

    use crate::lexer::{
        comment::CommentType, num::NumberLiteralValue, operator::OperatorType,
        punctuation::PunctuationType,
    };

    use super::*;

    #[test]
    fn test_file_tokenization() -> Result<()> {
        let src = r#"
// This is a a comment
const a = `my template: ${b}`;

function foo() {
    return /hello/gm.test("\u0041BC") == true && 1.2e-3;
}
"#;

        assert_eq!(
            tokenize(src).unwrap(),
            vec![
                Token::Comment(Comment::new(CommentType::Line(
                    " This is a a comment".to_string()
                ))),
                Token::Keyword(Keyword::new("const".try_into().into_diagnostic()?)),
                Token::Ident("a".into()),
                Token::Operator(Operator::new(OperatorType::Assignment)),
                Token::TemplateLiteralString(TemplateLiteralString::new(
                    "my template: ".into(),
                    false
                )),
                Token::TemplateLiteralExprOpen(TemplateLiteralExprOpen::default()),
                Token::Ident("b".into()),
                Token::TemplateLiteralExprClose(TemplateLiteralExprClose::default()),
                Token::TemplateLiteralString(TemplateLiteralString::new("".into(), true)),
                Token::Punctuation(Punctuation::new(PunctuationType::Semicolon)),
                Token::Keyword(Keyword::new("function".try_into().into_diagnostic()?)),
                Token::Ident("foo".into()),
                Token::Punctuation(Punctuation::new(PunctuationType::OpenParen)),
                Token::Punctuation(Punctuation::new(PunctuationType::CloseParen)),
                Token::Punctuation(Punctuation::new(PunctuationType::OpenBrace)),
                Token::Keyword(Keyword::new("return".try_into().into_diagnostic()?)),
                Token::RegexLiteral(RegexLiteral::new("hello".into(), "gm".into())),
                Token::Punctuation(Punctuation::new(PunctuationType::Dot)),
                Token::Ident("test".into()),
                Token::Punctuation(Punctuation::new(PunctuationType::OpenParen)),
                Token::StringLiteral(StringLiteral::new("ABC".into())),
                Token::Punctuation(Punctuation::new(PunctuationType::CloseParen)),
                Token::Operator(Operator::new(OperatorType::LooseEquality)),
                Token::ValueLiteral(ValueLiteral::new("true".try_into().into_diagnostic()?)),
                Token::Operator(Operator::new(OperatorType::LogicalAnd)),
                Token::NumericLiteral(NumberLiteral::new(NumberLiteralValue::Primitive(1.2e-3))),
                Token::Punctuation(Punctuation::new(PunctuationType::Semicolon)),
                Token::Punctuation(Punctuation::new(PunctuationType::CloseBrace)),
            ]
        );

        Ok(())
    }
}
