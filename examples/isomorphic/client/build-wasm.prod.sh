#!/bin/bash

# cd to the root directory of this repository
cd $(dirname $0)

rm -rf dist/
mkdir -p build/

cargo +nightly build -p isomorphic-client --release --target wasm32-unknown-unknown &&
  wasm-bindgen --no-typescript ../../../target/wasm32-unknown-unknown/release/isomorphic_client.wasm --out-dir ./build &&
  NODE_ENV=production ../../../node_modules/webpack-cli/bin/cli.js
