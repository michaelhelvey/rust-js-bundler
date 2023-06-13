use miette::{Diagnostic, NamedSource, SourceSpan};
use thiserror::Error;

// TODO: use the location and position thrown by a given error to create a
// miette diagnostic.

/// Represents an error that occurred while parsing or lexing a source file.
/// Every error has a specific character at which the parser decides to throw an
/// error, and this is represented by the "point" field.  Many errors also have
/// a range or token that causes the error to be thrown, (such as "SyntaxError:
/// unexpected token '}' expected Keyword").
#[derive(Error, Debug, Diagnostic)]
#[error("SyntaxError")]
pub struct SyntaxError {
    #[source_code]
    pub src: NamedSource,
    #[label("Unexpected token")]
    pub span: SourceSpan,
}
