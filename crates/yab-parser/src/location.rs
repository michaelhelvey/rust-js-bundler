use serde::Serialize;

/// Represents the position of a single character in a source file.
#[derive(Debug, Serialize, PartialEq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub index: usize,
}

/// Represents the location of a token in a source file.
#[derive(Debug, Serialize, PartialEq)]
pub struct Location {
    pub start: Position,
    pub end: Position,
    pub file_path: String,
}
