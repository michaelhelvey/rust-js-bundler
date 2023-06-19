use miette::{miette, ErrReport, LabeledSpan, Severity, SourceSpan};
use serde::Serialize;

/// Represents the position of a single character in a source file.
#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub index: usize,
}

/// Represents the location of a token in a source file.
#[derive(Debug, Serialize, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
    pub file_path: String,
}

impl Into<SourceSpan> for Span {
    fn into(self) -> SourceSpan {
        SourceSpan::new(self.start.index.into(), self.end.index.into())
    }
}

/// Custom iterator over the characters in a string of source code.  Provides
/// functionality not otherwise available in the standard library's collection
/// of iterators, such as multi-character lookahead, location tracing, and error
/// reporting integration with miette, our diagnostic library of choice.
#[derive(Debug)]
pub struct CodeIter {
    current_pos: Position,
    source: String,
    file_path: String,
    chars: Vec<char>,
}

pub trait IntoCodeIterator {
    fn into_code_iterator<'chars>(self, file_path: String) -> CodeIter;
}

impl IntoCodeIterator for String {
    /// Consumes the string to create an iterator over its characters.
    fn into_code_iterator(self, file_path: String) -> CodeIter {
        CodeIter {
            current_pos: Position {
                line: 1,
                column: 1,
                index: 0,
            },
            chars: self.chars().collect::<Vec<char>>(),
            source: self,
            file_path,
        }
    }
}

impl IntoCodeIterator for &str {
    /// Consumes the string to create an iterator over its characters.
    fn into_code_iterator(self, file_path: String) -> CodeIter {
        CodeIter {
            current_pos: Position {
                line: 1,
                column: 1,
                index: 0,
            },
            chars: self.chars().collect::<Vec<char>>(),
            source: self.to_string(),
            file_path,
        }
    }
}

impl Iterator for CodeIter {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        let char = self.chars.get(self.current_pos.index);

        match char {
            Some('\n') => {
                self.current_pos.index += 1;
                self.current_pos.line += 1;
                self.current_pos.column = 1;
                Some('\n')
            }
            Some(c) => {
                self.current_pos.index += 1;
                self.current_pos.column += 1;
                Some(*c)
            }
            None => return None,
        }
    }
}

impl CodeIter {
    /// Returns the next character in the iterator without consuming it.
    pub fn peek(&self) -> Option<&char> {
        self.chars.get(self.current_pos.index)
    }

    /// Returns the character `n` characters ahead in the iterator without
    /// consuming it.  peek_forward(0) is equivalent to peek().
    pub fn peek_forward(&self, n: usize) -> Option<&char> {
        self.chars.get(self.current_pos.index + n)
    }

    /// Returns the current position of the iterator, expressed as a `Position`
    /// struct.
    pub fn current_position(&self) -> Position {
        self.current_pos.clone()
    }

    /// Given a start position, constructs a `Location` struct representing the
    /// location that a given token spans.
    pub fn location_from_start(&self, start: Position) -> Span {
        let end = self.current_pos.clone();
        Span {
            start,
            end,
            file_path: self.file_path.clone(),
        }
    }

    /// Creates a miette `ErrReport` from a given `Span`
    pub fn to_span_error(&self, err_msg: &str, location: Span) -> ErrReport {
        miette!(
            severity = Severity::Error,
            code = "SyntaxError",
            labels = vec![LabeledSpan::at(location, err_msg)],
            "{}",
            err_msg
        )
        // This could be quite expensive on very large files, and I don't love
        // that, but I'm not sure if there's a better way to do it without
        // turning this trait into lifetime soup. I think I'm generally ok with
        // creating errors to be expensive, since it means we're terminating the
        // process, though?
        .with_source_code(self.source.clone())
    }

    /// Creates a miette `ErrReport` from a given `Position`.
    pub fn to_position_error(&self, err_msg: &str, position: Position) -> ErrReport {
        miette!(
            severity = Severity::Error,
            code = "SyntaxError",
            labels = vec![LabeledSpan::at_offset(position.index, err_msg)],
            "{}",
            err_msg
        )
        .with_source_code(self.source.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iter() {
        let src = "ab".to_string();
        let mut iter = src.into_code_iterator("foo.js".into());

        assert_eq!(iter.next().unwrap(), 'a');
        assert_eq!(iter.next().unwrap(), 'b');
        assert_eq!(iter.next().is_none(), true);
    }

    #[test]
    fn test_peek() {
        let src = "ab".to_string();
        let mut iter = src.into_code_iterator("foo.js".into());
        assert_eq!(iter.peek().unwrap(), &'a');
        assert_eq!(iter.peek().unwrap(), &'a');

        _ = iter.next();
        _ = iter.next();

        assert_eq!(iter.peek().is_none(), true);
    }

    #[test]
    fn test_peek_multi() {
        let src = "abc".to_string();
        let mut iter = src.into_code_iterator("foo.js".into());
        assert_eq!(iter.peek_forward(2), Some(&'c'));
        assert_eq!(iter.peek_forward(3), None);
    }
}
