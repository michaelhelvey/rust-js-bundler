#!/usr/bin/env bash

# Sanity checks that the parser can do useful things on real-world code.

set -eou pipefail

# get current directory
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

curl https://cdn.jsdelivr.net/npm/react-dom/cjs/react-dom.development.js > $DIR/react-dom.development.js

cargo run --bin lex -p yab-parser $DIR/react-dom.development.js
