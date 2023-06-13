use miette::NamedSource;
use miette::Result;
use yab_parser::error::SyntaxError;

fn fallible() -> Result<()> {
    let src = "const a = b;";

    Err(SyntaxError {
        src: NamedSource::new("test.js".to_string(), src.to_string()),
        span: (3, 2).into(),
    })?;

    Ok(())
}

pub fn main() -> Result<()> {
    // Read a file from stdin, lex it, and print the tokens in a serialized format
    // let input_file_path = std::env::args().nth(1).unwrap();
    // let input = std::fs::read_to_string(input_file_path).unwrap();
    // let tokens = yab_parser::lexer::tokenize(&input).unwrap();

    // let pretty_tokens = serde_json::to_string_pretty(&tokens).unwrap();
    // println!("{}", pretty_tokens)

    fallible()?;

    Ok(())
}
