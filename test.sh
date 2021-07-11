#!/bin/bash

cd $(git rev-parse --show-toplevel)

cargo check --all && # Make sure examples compile
cargo test --all &&
wasm-pack test --firefox --headless crates/virtual-dom-rs 
