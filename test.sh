#!/bin/bash

set -e

cd $(git rev-parse --show-toplevel)

cargo check --all
cargo test --all

# Ensure that `crates/virtual-dom` can compile without web-sys/js-sys/wasm-bindgen.
cargo check -p virtual-node --no-default-features

wasm-pack test --firefox --headless crates/percy-dom
