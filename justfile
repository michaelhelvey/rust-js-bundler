default:
  cargo run -- --entrypoint ./js/example/src/index.js

test_parser:
  cargo nextest run -p yab-parser

example:
  node ./example/src/index.js
