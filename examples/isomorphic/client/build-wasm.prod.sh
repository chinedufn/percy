#!/bin/bash

cd $(dirname $0)

rm -rf dist/
mkdir -p dist/

cargo +nightly build -p isomorphic-client --release --target wasm32-unknown-unknown &&
  wasm-bindgen --no-modules --no-typescript ../../../target/wasm32-unknown-unknown/release/isomorphic_client.wasm --out-dir ./dist &&
  NODE_ENV=production ../../../node_modules/webpack-cli/bin/cli.js
