#!/bin/bash

cd $(git rev-parse --show-toplevel)

OUTPUT_CSS=examples/css-in-rust/app.css cargo run -p css-in-rust
