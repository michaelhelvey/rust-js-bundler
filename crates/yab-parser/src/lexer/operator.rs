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

    #[token(lexeme = "*")]
    #[strum(serialize = "*")]
    Multiplication,

    #[token(lexeme = "/")]
    #[strum(serialize = "/")]
    Division,

    #[token(lexeme = "**")]
    #[strum(serialize = "**")]
    Exponentiation,

    #[token(lexeme = "%")]
    #[strum(serialize = "%")]
    Modulo,

    #[token(lexeme = "++")]
    #[strum(serialize = "++")]
    Increment,

    #[token(lexeme = "--")]
    #[strum(serialize = "--")]
    Decrement,

    #[token(lexeme = "=")]
    #[strum(serialize = "=")]
    Assignment,

    #[token(lexeme = "*=")]
    #[strum(serialize = "*=")]
    MultiplicationAssignment,

    #[token(lexeme = "/=")]
    #[strum(serialize = "/=")]
    DivisionAssignment,

    #[token(lexeme = "+=")]
    #[strum(serialize = "+=")]
    AdditionAssignment,

    #[token(lexeme = "-=")]
    #[strum(serialize = "-=")]
    SubtractionAssigment,

    #[token(lexeme = "<<=")]
    #[strum(serialize = "<<=")]
    ShiftLeftAssignment,

    #[token(lexeme = ">>=")]
    #[strum(serialize = ">>=")]
    ShiftRightAssignment,

    #[token(lexeme = ">>>=")]
    #[strum(serialize = ">>>=")]
    ShiftRightUnsignedAssignment,

    #[token(lexeme = "&=")]
    #[strum(serialize = "&=")]
    BitwiseAndAssignment,

    #[token(lexeme = "|=")]
    #[strum(serialize = "|=")]
    BitwiseOrAssignment,

    #[token(lexeme = "^=")]
    #[strum(serialize = "^=")]
    BitwiseXOrAssignment,

    #[token(lexeme = "&&=")]
    #[strum(serialize = "&&=")]
    LogicalAndAssignment,

    #[token(lexeme = "||=")]
    #[strum(serialize = "||=")]
    LogicalOrAssignment,

    #[token(lexeme = "??=")]
    #[strum(serialize = "??=")]
    NullishCoalescingAssignment,

    #[token(lexeme = "==")]
    #[strum(serialize = "==")]
    LooseEquality,

    #[token(lexeme = "!=")]
    #[strum(serialize = "!=")]
    LooseNotEquality,

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

    #[token(lexeme = "||")]
    #[strum(serialize = "||")]
    LogicalOr,

    #[token(lexeme = "??")]
    #[strum(serialize = "??")]
    NullishCoalescing,

    #[token(lexeme = "~")]
    #[strum(serialize = "~")]
    BitwiseNot,

    #[token(lexeme = "&")]
    #[strum(serialize = "&")]
    BitwiseAnd,

    #[token(lexeme = "|")]
    #[strum(serialize = "|")]
    BitwiseOr,

    #[token(lexeme = "^")]
    #[strum(serialize = "^")]
    BitwiseXOr,

    #[token(lexeme = "<<")]
    #[strum(serialize = "<<")]
    BitwiseShiftLeft,

    #[token(lexeme = ">>")]
    #[strum(serialize = ">>")]
    BitwiseShiftRight,

    #[token(lexeme = ">>>")]
    #[strum(serialize = ">>>")]
    BitwiseShiftRightUnsigned,

    #[token(lexeme = ">")]
    #[strum(serialize = ">")]
    GreaterThan,

    #[token(lexeme = ">=")]
    #[strum(serialize = ">=")]
    GreaterThanOrEqualTo,

    #[token(lexeme = "<")]
    #[strum(serialize = "<")]
    LessThan,

    #[token(lexeme = "<=")]
    #[strum(serialize = "<=")]
    LessThanOrEqualTo,

    #[token(lexeme = "?")]
    #[strum(serialize = "?")]
    Ternary,

    #[token(lexeme = "...")]
    #[strum(serialize = "...")]
    ObjectSpread,

    #[token(lexeme = "await")]
    #[strum(serialize = "await")]
    Await,

    #[token(lexeme = "void")]
    #[strum(serialize = "void")]
    Void,

    #[token(lexeme = "typeof")]
    #[strum(serialize = "typeof")]
    TypeOf,

    #[token(lexeme = "instanceof")]
    #[strum(serialize = "instanceof")]
    InstanceOf,

    #[token(lexeme = "in")]
    #[strum(serialize = "in")]
    In,

    #[token(lexeme = "yield")]
    #[strum(serialize = "yield")]
    Yield,
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
            ("await", OperatorType::Await),
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
