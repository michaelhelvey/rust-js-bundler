default:
  cargo run -- --entrypoint ./js/example/src/index.js

test_lexer *args:
  cargo nextest run lexer::{{args}} -p yab-parser

example:
  node ./example/src/index.js
