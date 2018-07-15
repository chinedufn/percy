#!/bin/bash

cd $(git rev-parse --show-toplevel)

cargo +nightly build -p jsdom-tests --target wasm32-unknown-unknown && \
  wasm-bindgen --nodejs --no-typescript target/wasm32-unknown-unknown/debug/jsdom_tests.wasm --out-dir ./tests/jsdom
