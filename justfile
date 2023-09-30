test:
  cargo nextest run

build:
  cargo build --release

test_lexer *args:
  cargo nextest run lexer::{{args}} -p yab-parser

bench:
  ./test/lex_bench.sh
