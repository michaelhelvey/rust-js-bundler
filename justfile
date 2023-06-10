default:
  cargo run -- --entrypoint ./js/example/src/index.js

test_parser:
  cargo nextest run number -p yab-parser

example:
  node ./example/src/index.js
