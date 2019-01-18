#!/bin/bash

cd $(git rev-parse --show-toplevel)

cargo build --all && # Make sure examples compile
cargo test --all &&
wasm-pack test crates/virtual-dom-rs --firefox --headless
