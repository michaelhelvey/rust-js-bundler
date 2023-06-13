default:
  cargo run -- --entrypoint ./js/example/src/index.js

test:
  cargo nextest run

test_lexer *args:
  cargo nextest run lexer::{{args}} -p yab-parser

example:
  node ./example/src/index.js
