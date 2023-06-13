pub fn main() {
    // Read a file from stdin, lex it, and print the tokens in a serialized format
    let input_file_path = std::env::args().nth(1).unwrap();
    let input = std::fs::read_to_string(input_file_path).unwrap();

    // TODO: lex the input
    // let tokens = yab_parser::lexer::lex(&input).unwrap();
}
