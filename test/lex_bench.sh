#!/usr/bin/env bash

# Sanity checks that the parser can do useful things on real-world code.

set -eou pipefail

# get current directory
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd )"

# if not file exists:
if [ ! -f "$DIR/react-dom.development.js" ]; then
    echo "Downloading react-dom.development.js"
    curl -q https://cdn.jsdelivr.net/npm/react-dom/cjs/react-dom.development.js > $DIR/react-dom.development.js
fi

cargo build --release --bin lex-bench -p yab-parser

# warmup filesystem caches required to load the first argument efficiently:
hyperfine --warmup 3 "./target/release/lex-bench $DIR/react-dom.development.js"
