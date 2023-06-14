use miette::IntoDiagnostic;
use miette::Result;

pub fn main() -> Result<()> {
    let input_file_path = std::env::args().nth(1).unwrap();
    let input = std::fs::read_to_string(input_file_path).into_diagnostic()?;
    let tokens = yab_parser::lexer::tokenize(&input)?;

    let pretty_tokens = serde_json::to_string_pretty(&tokens).unwrap();
    println!("{}", pretty_tokens);

    Ok(())
}
