use color_eyre::{eyre::eyre, Result};
use std::{iter::Peekable, str::Chars};

// TODO:
// *) template literals
// *) operators
// *) regex literals
// *) punctuators
// *) hashbang comments
mod comment;
mod escape_chars;
mod ident;
mod num;
mod old_number;
mod string;
mod token;
mod utils;

use old_number::parse_number;
use token::{
    HasPrefixLookup, Ident, Keyword, KeywordType, NumberLiteral, Operator, OperatorType,
    Punctuation, PunctuationType, StringLiteral, Token, ValueLiteral, ValueLiteralType,
};

macro_rules! tokenize_prefix {
    ($token_type:ident, $enum_type:ident, $current_char:ident, $chars:ident, $tokens:ident, $cont_label:lifetime) => {
        // There doesn't seem to be a way to get a slice out of an iterator, so
        // for now we will just allocate :/
        let mut prefix_lexeme = String::from($current_char);
        let mut prefix_matches = $enum_type::fields_starting_with(&prefix_lexeme);

        if prefix_matches > 0 {
            'prefix: while let Some(next_char) = $chars.peek() {
                prefix_lexeme.push(*next_char);
                prefix_matches = $enum_type::fields_starting_with(&prefix_lexeme);

                if prefix_matches == 0 {
                    prefix_lexeme = prefix_lexeme[0..prefix_lexeme.len() - 1].to_string();
                    break 'prefix;
                } else {
                    _ = $chars.next()
                }
            }

            let prefix_ref: &str = prefix_lexeme.as_ref();
            let prefix: $enum_type = prefix_ref.try_into().expect(&format!(
                "Internal tokenizer error: could not parse {} into $token_type",
                prefix_ref
            ));

            $tokens.push(Token::$token_type($token_type::new(prefix)));
            continue $cont_label;
        }
    };
}

fn is_line_separator(c: char) -> bool {
    matches!(c, '\u{000A}' | '\u{000D}' | '\u{2028}' | '\u{2029}')
}

fn parse_escape_character(chars: &mut Peekable<Chars>) -> Result<Option<char>> {
    match chars.next() {
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
    }
}

pub fn tokenize(source: &str) -> Result<Vec<Token>> {
    let mut chars = source.chars().peekable();
    let mut tokens = Vec::<Token>::new();
    let mut in_template_literal = false;

    'outer: while let Some(current_char) = chars.next() {
        if is_line_separator(current_char) && in_template_literal {
            // line separator in a template literal expression block:
            return Err(eyre!(
                "Unexpected line terminator while parsing template literal expression"
            ));
        }

        if current_char.is_whitespace() {
            continue 'outer;
        }

        // if the current character is (optionally) +/- followed by digit,
        // (or simply a digit) start trying to parse the remaining tokens as a
        // number, until we reach whitespace.  Then pass the result to our real
        // number parser to create the number -- if it's not a valid number,
        // then it's not a valid identifier anyway because it starts with a
        // number, so the number parsing error is the appropriate thing to
        // return.
        let mut could_be_number =
            matches!(current_char, '+' | '-') && matches!(chars.peek(), Some('0'..='9'));
        could_be_number = could_be_number || current_char.is_ascii_digit();
        if could_be_number {
            let mut lexeme = String::from(current_char);
            for next_char in chars.by_ref() {
                if matches!(next_char, '0'..='9' | 'a'..='f' | 'A'..='F' | '.' | '_' | '+' | '-' | 'o' | 'O' | 'x' | 'X')
                {
                    lexeme.push(next_char);
                } else {
                    break;
                }
            }
            let number_value = parse_number(lexeme.as_ref(), true)?;
            tokens.push(Token::NumberLiteral(NumberLiteral::new(number_value)));
            continue 'outer;
        }

        if current_char == '/' {
            // Regexes are allowed to begin with any unicode code point EXCEPT
            // line terminators, *, /, \, and [

            if let Some(next_char) = chars.peek() {
                if !matches!(next_char, '/' | '*' | '[' | '\\') && !is_line_separator(*next_char) {
                    let mut lexeme = String::from(current_char);
                    'regex: for next_char in chars.by_ref() {
                        lexeme.push(next_char);

                        if next_char == '/' {
                            while let Some(flag_char) = chars.peek() {
                                if flag_char.is_alphabetic() {
                                    lexeme.push(*flag_char);
                                    _ = chars.next();
                                } else {
                                    break 'regex;
                                }
                            }
                            break 'regex;
                        }
                    }

                    tokens.push(Token::RegexLiteral(StringLiteral::from(lexeme)));

                    continue 'outer;
                }
            }
        }

        if current_char == '/' && matches!(chars.peek(), Some('/')) {
            // single line comment
            while let Some(next_char) = chars.next() {
                if is_line_separator(next_char) {
                    break;
                }
            }
            continue 'outer;
        }

        // Hashbang comments are only allowed at the beginning of the file:
        if current_char == '#' && matches!(chars.peek(), Some('!')) && tokens.is_empty() {
            // hashbang comment
            while let Some(next_char) = chars.next() {
                if is_line_separator(next_char) {
                    break;
                }
            }
            continue 'outer;
        }

        if current_char == '/' && matches!(chars.peek(), Some('*')) {
            // multi-line comment
            let mut next_char = chars.next().unwrap();
            while let Some(next_next_char) = chars.next() {
                if next_char == '*' && next_next_char == '/' {
                    break;
                }
                next_char = next_next_char;
            }
            continue 'outer;
        }

        if matches!(current_char, '\'' | '"') {
            let unexpected_eof_msg = "Unexpected EOF while parsing string";
            if chars.peek().is_none() {
                return Err(eyre!(unexpected_eof_msg));
            }
            let mut lexeme = String::new();
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
                    let escaped_char = parse_escape_character(&mut chars);
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

            tokens.push(Token::StringLiteral(lexeme.into()));
            continue 'outer;
        }

        // Template Literals:
        if current_char == '`' {
            in_template_literal = true;

            if chars.peek().is_none() {
                return Err(eyre!("Unexpected EOF while parsing template literal"));
            }

            let mut content = String::from(current_char);
            let mut found_end = false;

            'string: while let Some(next_char) = chars.next() {
                if next_char == '\\' {
                    let escaped_char = parse_escape_character(&mut chars);
                    if let Some(c) = escaped_char? {
                        content.push(c);
                    }

                    continue 'string;
                }

                content.push(next_char);
                if next_char == '$' && matches!(chars.peek(), Some('{')) {
                    // safety: we just peeked it
                    let bracket_char = chars.next().unwrap();
                    content.push(bracket_char);
                    break 'string;
                }

                if next_char == '`' {
                    found_end = true;
                    break 'string;
                }
            }

            tokens.push(Token::TemplateLiteralOpen(StringLiteral::from(content)));

            if found_end {
                tokens.push(Token::TemplateLiteralClose(StringLiteral::from(
                    String::from("`"),
                )));
                in_template_literal = false;
            }

            continue 'outer;
        }

        if current_char == '}' && in_template_literal {
            // read until we get to either the end of the template literal or the next ${
            let mut found_end = false;
            let mut content = String::from(current_char);
            'string: while let Some(next_char) = chars.next() {
                if next_char == '\\' {
                    let escaped_char = parse_escape_character(&mut chars);
                    if let Some(c) = escaped_char? {
                        content.push(c);
                    }

                    continue 'string;
                }

                content.push(next_char);
                if next_char == '$' && matches!(chars.peek(), Some('{')) {
                    // safety: we just peeked it
                    let bracket_char = chars.next().unwrap();
                    content.push(bracket_char);
                    break;
                }

                if next_char == '`' {
                    found_end = true;
                    break;
                }
            }

            if chars.peek().is_none() && !found_end {
                return Err(eyre!("Unexpected EOF while parsing template literal"));
            }

            if found_end {
                in_template_literal = false;
                tokens.push(Token::TemplateLiteralClose(StringLiteral::from(content)));
            } else {
                tokens.push(Token::TemplateLiteralContent(StringLiteral::from(content)));
            }

            continue 'outer;
        }

        if current_char.is_alphabetic() {
            let mut lexeme = String::from(current_char);

            // TODO: support unicode escape sequences in identifiers
            'ident: while let Some(next_char) = chars.peek() {
                if next_char.is_alphabetic() {
                    lexeme.push(*next_char);
                    _ = chars.next();
                } else {
                    break 'ident;
                }
            }

            if let Ok(keyword_type) = KeywordType::try_from(lexeme.as_str()) {
                tokens.push(Token::Keyword(Keyword::new(keyword_type)));
            } else if let Ok(value) = ValueLiteralType::try_from(lexeme.as_str()) {
                tokens.push(Token::ValueLiteral(ValueLiteral::new(value)));
            } else {
                tokens.push(Token::Ident(Ident::from(lexeme)))
            }

            continue 'outer;
        }

        tokenize_prefix!(Operator, OperatorType, current_char, chars, tokens, 'outer);
        tokenize_prefix!(Punctuation, PunctuationType, current_char, chars, tokens, 'outer);

        return Err(eyre!("Unexpected character: {}", current_char));
    }

    if in_template_literal {
        return Err(eyre!("Unexpected EOF while parsing template literal"));
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;
    use color_eyre::Result;

    #[test]
    fn test_macro_prefix_lookup() -> Result<()> {
        assert_eq!(OperatorType::fields_starting_with("="), 3);
        assert_eq!(OperatorType::fields_starting_with("=="), 2);
        assert_eq!(OperatorType::fields_starting_with("==="), 1);
        assert_eq!(OperatorType::fields_starting_with("~!~~"), 0);

        Ok(())
    }

    #[test]
    fn operator_tokenization() -> Result<()> {
        assert_eq!(
            tokenize("=")?,
            vec![Token::Operator(Operator::new(OperatorType::Assignment))]
        );
        assert_eq!(
            tokenize("==")?,
            vec![Token::Operator(Operator::new(OperatorType::LooseEquality))]
        );
        assert_eq!(
            tokenize("===")?,
            vec![Token::Operator(Operator::new(OperatorType::StrictEquality))]
        );
        assert_eq!(
            tokenize("+")?,
            vec![Token::Operator(Operator::new(OperatorType::Plus))]
        );
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
    fn tokenize_boolean_literal() -> Result<()> {
        let src = "true";

        assert_eq!(
            tokenize(src)?,
            vec![Token::ValueLiteral(ValueLiteral::new(
                ValueLiteralType::True
            ))]
        );

        let src = "false";
        assert_eq!(
            tokenize(src)?,
            vec![Token::ValueLiteral(ValueLiteral::new(
                ValueLiteralType::False
            ))]
        );

        Ok(())
    }

    #[test]
    fn tokenize_null_literal() -> Result<()> {
        let src = "null";
        assert_eq!(
            tokenize(src)?,
            vec![Token::ValueLiteral(ValueLiteral::new(
                ValueLiteralType::Null
            ))]
        );

        Ok(())
    }

    #[test]
    fn test_single_line_comments() -> Result<()> {
        let src = r#"
// this is a comment
const // also a comment
"#;
        assert_eq!(
            tokenize(src)?,
            vec![Token::Keyword(Keyword::new(KeywordType::Const))]
        );
        Ok(())
    }

    #[test]
    fn test_hashbang() -> Result<()> {
        let src = r#"
#!/usr/bin/env node
const
"#;
        assert_eq!(
            tokenize(src)?,
            vec![Token::Keyword(Keyword::new(KeywordType::Const))]
        );
        Ok(())
    }

    #[test]
    fn test_hashbang_not_at_beginning_of_file() -> Result<()> {
        let src = r#"
const
#!/usr/bin/env node
"#;
        let result = tokenize(src);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err().to_string(), "Unexpected character: #");

        Ok(())
    }

    #[test]
    fn test_multi_line_comments() -> Result<()> {
        let src = r#"
/*
 * this is a comment
 */
const /* more comments */
"#;
        assert_eq!(
            tokenize(src)?,
            vec![Token::Keyword(Keyword::new(KeywordType::Const))]
        );
        Ok(())
    }

    #[test]
    fn test_template_literal() -> Result<()> {
        let src = r#"`hello ${world} abc ${foo} bar`"#;
        assert_eq!(
            tokenize(src)?,
            vec![
                Token::TemplateLiteralOpen(StringLiteral::from("`hello ${")),
                Token::Ident(Ident::from("world".to_string())),
                Token::TemplateLiteralContent(StringLiteral::from("} abc ${")),
                Token::Ident(Ident::from("foo".to_string())),
                Token::TemplateLiteralClose(StringLiteral::from("} bar`"))
            ]
        );
        Ok(())
    }

    #[test]
    fn test_template_literals_follow_same_escaping_rules() -> Result<()> {
        let src = r#"`hello \`
${world}`"#;

        assert_eq!(
            tokenize(src)?,
            vec![
                Token::TemplateLiteralOpen(StringLiteral::from("`hello `\n${")),
                Token::Ident(Ident::from("world".to_string())),
                Token::TemplateLiteralClose(StringLiteral::from("}`")),
            ]
        );
        Ok(())
    }

    #[test]
    fn test_template_literals_unexpected_eof_after_expr_close() {
        let src = r#"`hello ${world}"#;
        let result = tokenize(src);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unexpected EOF while parsing template literal"
        );
    }

    #[test]
    fn test_template_literals_unexpected_eof_before_expr_close() {
        let src = r#"`hello ${world"#;
        let result = tokenize(src);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unexpected EOF while parsing template literal"
        );
    }

    #[test]
    fn test_template_literals_unexpected_eof_before_content() {
        let src = r#"`"#;
        let result = tokenize(src);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unexpected EOF while parsing template literal"
        );
    }

    #[test]
    fn test_template_literals_unexpected_lf_in_expr() {
        let src = r#"
`hello ${world
}`
        "#;
        let result = tokenize(src);
        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Unexpected line terminator while parsing template literal expression"
        );
    }

    #[test]
    fn test_simple_regex_without_flags() -> Result<()> {
        let src = r#"/hello/"#;
        assert_eq!(
            tokenize(src)?,
            vec![Token::RegexLiteral(StringLiteral::from(
                "/hello/".to_string(),
            ))]
        );
        Ok(())
    }

    #[test]
    fn test_regex_with_flags() -> Result<()> {
        let src = r#"/hello/gmi"#;
        assert_eq!(
            tokenize(src)?,
            vec![Token::RegexLiteral(StringLiteral::from(
                "/hello/gmi".to_string(),
            ))]
        );
        Ok(())
    }

    #[test]
    fn test_integers() -> Result<()> {
        let src = r#"123"#;
        assert_eq!(tokenize(src)?, vec![Token::NumberLiteral(123.into())]);
        Ok(())
    }

    #[test]
    fn test_hex_values() -> Result<()> {
        let src = r#"0xFF"#;
        assert_eq!(tokenize(src)?, vec![Token::NumberLiteral(255.into())]);
        Ok(())
    }

    #[test]
    fn sanity_check_large_expression() -> Result<()> {
        let src = r#"
const a = `my template: ${b}`;

function foo() {
    return /hello/gm.test("\u0041BC");
}
"#;

        assert_eq!(
            tokenize(src)?,
            vec![
                Token::Keyword(Keyword::new("const".try_into()?)),
                Token::Ident(Ident::from("a".to_string())),
                Token::Operator(Operator::new(OperatorType::Assignment)),
                Token::TemplateLiteralOpen(StringLiteral::from("`my template: ${")),
                Token::Ident(Ident::from("b".to_string())),
                Token::TemplateLiteralClose(StringLiteral::from("}`")),
                Token::Punctuation(Punctuation::new(PunctuationType::Semicolon)),
                Token::Keyword(Keyword::new("function".try_into()?)),
                Token::Ident(Ident::from("foo".to_string())),
                Token::Punctuation(Punctuation::new(PunctuationType::OpenParen)),
                Token::Punctuation(Punctuation::new(PunctuationType::CloseParen)),
                Token::Punctuation(Punctuation::new(PunctuationType::OpenBrace)),
                Token::Keyword(Keyword::new("return".try_into()?)),
                Token::RegexLiteral(StringLiteral::from("/hello/gm".to_string())),
                Token::Punctuation(Punctuation::new(PunctuationType::Dot)),
                Token::Ident(Ident::from("test".to_string())),
                Token::Punctuation(Punctuation::new(PunctuationType::OpenParen)),
                Token::StringLiteral(StringLiteral::from("ABC".to_string())),
                Token::Punctuation(Punctuation::new(PunctuationType::CloseParen)),
                Token::Punctuation(Punctuation::new(PunctuationType::Semicolon)),
                Token::Punctuation(Punctuation::new(PunctuationType::CloseBrace)),
            ]
        );

        Ok(())
    }
}
