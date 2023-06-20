use miette::{miette, ErrReport, LabeledSpan, NamedSource, Severity, SourceSpan};
use serde::Serialize;

/// Represents the position of a single character in a source file.
#[derive(Debug, Serialize, PartialEq, Clone)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub index: usize,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            line: 1,
            column: 1,
            index: 0,
        }
    }
}

/// Represents the location of a token in a source file.
#[derive(Debug, Serialize, PartialEq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
    pub file_path: String,
}

impl Span {
    pub fn new(start: Position, end: Position, file_path: impl Into<String>) -> Self {
        Self {
            start,
            end,
            file_path: file_path.into(),
        }
    }
}

impl Into<SourceSpan> for Span {
    fn into(self) -> SourceSpan {
        SourceSpan::new(
            self.start.index.into(),
            (self.end.index - self.start.index).into(),
        )
    }
}

/// Custom iterator over the characters in a string of source code.  Provides
/// functionality not otherwise available in the standard library's collection
/// of iterators, such as multi-character lookahead, location tracing, and error
/// reporting integration with miette, our diagnostic library of choice.
#[derive(Debug)]
pub struct CodeIter {
    current_position: Position,
    previous_position: Option<Position>,
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
            current_position: Position {
                line: 1,
                column: 1,
                index: 0,
            },
            previous_position: None,
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
            current_position: Position {
                line: 1,
                column: 1,
                index: 0,
            },
            previous_position: None,
            chars: self.chars().collect::<Vec<char>>(),
            source: self.to_string(),
            file_path,
        }
    }
}

impl Iterator for CodeIter {
    type Item = char;

    fn next(&mut self) -> Option<Self::Item> {
        self.previous_position = Some(self.current_position.clone());
        let char = self.chars.get(self.current_position.index);

        match char {
            Some('\n') => {
                self.current_position.index += 1;
                self.current_position.line += 1;
                self.current_position.column = 1;
                Some('\n')
            }
            Some(c) => {
                self.current_position.index += 1;
                self.current_position.column += 1;
                Some(*c)
            }
            None => return None,
        }
    }
}

impl CodeIter {
    /// Returns the next character in the iterator without consuming it.
    pub fn peek(&self) -> Option<&char> {
        self.chars.get(self.current_position.index)
    }

    /// Returns the character `n` characters ahead in the iterator without
    /// consuming it.  peek_forward(0) is equivalent to peek().
    pub fn peek_forward(&self, n: usize) -> Option<&char> {
        self.chars.get(self.current_position.index + n)
    }

    /// Returns the current position of the iterator, expressed as a `Position`
    /// struct.
    pub fn current_position(&self) -> Position {
        self.current_position.clone()
    }

    pub fn previous_position(&self) -> Position {
        match self.previous_position {
            Some(ref position) => position.clone(),
            // If we haven't moved yet, we're at the beginning of the file.
            None => Position::default(),
        }
    }

    pub fn file_path(&self) -> &str {
        &self.file_path
    }

    /// Creates a miette `ErrReport` from a given `Span`
    pub fn to_span_error(&self, err_msg: &str, location: Span) -> ErrReport {
        let column = location.start.column;
        let line = location.start.line;

        self.to_error_with_label(err_msg, LabeledSpan::at(location, err_msg), line, column)
    }

    fn to_error_with_label(
        &self,
        err_msg: &str,
        label: LabeledSpan,
        line: usize,
        column: usize,
    ) -> ErrReport {
        miette!(
            severity = Severity::Error,
            code = "SyntaxError",
            labels = vec![label],
            "SyntaxError: {} at {}:{}:{}",
            err_msg,
            self.file_path,
            line,
            column
        )
        // This could be quite expensive on very large files, and I don't love
        // that, but I'm not sure if there's a better way to do it without
        // turning this trait into lifetime soup. I think I'm generally ok with
        // creating errors to be expensive, since it means we're terminating the
        // process, though?
        .with_source_code(NamedSource::new(
            self.file_path.clone(),
            self.source.clone(),
        ))
    }
}

macro_rules! current_span_error {
    ($iter:expr, $start:expr, $err_msg:literal, $($arg:tt)*) => {
        $iter.to_span_error(
            format!($err_msg, $($arg)*).as_str(),
            Span::new($start, $iter.current_position(), $iter.file_path()),
        )
    };
}

macro_rules! previous_span_error {
    ($iter:expr, $start:expr, $err_msg:literal, $($arg:tt)*) => {
        $iter.to_span_error(
            format!($err_msg, $($arg)*).as_str(),
            Span::new($start, $iter.previous_position(), $iter.file_path()),
        )
    };
}

pub(crate) use current_span_error;
pub(crate) use previous_span_error;

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
        let iter = src.into_code_iterator("foo.js".into());
        assert_eq!(iter.peek_forward(2), Some(&'c'));
        assert_eq!(iter.peek_forward(3), None);
    }
}
