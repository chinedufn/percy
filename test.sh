#!/bin/bash

cd $(git rev-parse --show-toplevel)

cargo build --all && # Make sure examples compile
cargo test --all &&
cargo test -p virtual-dom-rs --target wasm32-unknown-unknown
