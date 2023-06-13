use std::{iter::Peekable, str::Chars};

use super::utils::{try_parse_from_prefix_lookup, HasPrefixLookup};
use serde::Serialize;
use strum_macros::EnumString;
use yab_parser_macros::HasPrefixLookup;

#[derive(Debug, Serialize, PartialEq, HasPrefixLookup, EnumString)]
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
    #[token(lexeme = "&&")]
    #[strum(serialize = "&&")]
    LogicalAnd,
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

pub fn try_parse_operator(chars: &mut Peekable<Chars>) -> Option<Operator> {
    try_parse_from_prefix_lookup::<OperatorType>(chars).map(Operator::new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_operator() {
        let operators = vec![
            ("+", OperatorType::Plus),
            ("=", OperatorType::Assignment),
            ("==", OperatorType::LooseEquality),
            ("===", OperatorType::StrictEquality),
            ("&&", OperatorType::LogicalAnd),
        ];

        for op in operators {
            let mut chars = op.0.chars().peekable();
            let parsed = try_parse_operator(&mut chars).unwrap();
            assert_eq!(parsed.kind, op.1);
        }
    }

    #[test]
    fn test_non_existent_operator() {
        let mut chars = "foo".chars().peekable();
        let parsed = try_parse_operator(&mut chars);
        assert!(parsed.is_none());
    }
}
