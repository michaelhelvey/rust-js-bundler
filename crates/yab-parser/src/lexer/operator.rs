use super::{
    code_iter::CodeIter,
    utils::{try_parse_from_prefix_lookup, HasPrefixLookup},
};
use serde::Serialize;
use strum_macros::EnumString;
use yab_parser_macros::HasPrefixLookup;

#[derive(Debug, Serialize, PartialEq, HasPrefixLookup, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum OperatorType {
    #[token(lexeme = "+")]
    #[strum(serialize = "+")]
    Plus,

    #[token(lexeme = "-")]
    #[strum(serialize = "-")]
    Minus,

    #[token(lexeme = "=")]
    #[strum(serialize = "=")]
    Assignment,

    #[token(lexeme = "==")]
    #[strum(serialize = "==")]
    LooseEquality,

    #[token(lexeme = "===")]
    #[strum(serialize = "===")]
    StrictEquality,

    #[token(lexeme = "!==")]
    #[strum(serialize = "!==")]
    StrictNotEquality,

    #[token(lexeme = "!")]
    #[strum(serialize = "!")]
    LogicalNot,

    #[token(lexeme = "&&")]
    #[strum(serialize = "&&")]
    LogicalAnd,

    #[token(lexeme = ">")]
    #[strum(serialize = ">")]
    GreaterThan,

    #[token(lexeme = "<")]
    #[strum(serialize = "<")]
    LessThan,

    #[token(lexeme = "?")]
    #[strum(serialize = "?")]
    Ternary,
}

#[derive(Debug, Serialize, PartialEq)]
pub struct Operator {
    kind: OperatorType,
}

impl Operator {
    pub fn new(kind: OperatorType) -> Self {
        Self { kind }
    }
}

pub fn try_parse_operator(chars: &mut CodeIter) -> Option<Operator> {
    try_parse_from_prefix_lookup::<OperatorType>(chars).map(Operator::new)
}

#[cfg(test)]
mod tests {
    use crate::lexer::code_iter::IntoCodeIterator;

    use super::*;

    #[test]
    fn test_parse_operator() {
        let operators = vec![
            ("+", OperatorType::Plus),
            ("=", OperatorType::Assignment),
            ("==", OperatorType::LooseEquality),
            ("===", OperatorType::StrictEquality),
            ("&&", OperatorType::LogicalAnd),
            ("!==", OperatorType::StrictNotEquality),
        ];

        for op in operators {
            let mut chars = op.0.into_code_iterator("script.js".to_string());
            let parsed = try_parse_operator(&mut chars).unwrap();
            assert_eq!(parsed.kind, op.1);
        }
    }

    #[test]
    fn test_non_existent_operator() {
        let mut chars = "foo".into_code_iterator("script.js".to_string());
        let parsed = try_parse_operator(&mut chars);
        assert!(parsed.is_none());
    }
}
