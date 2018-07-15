#!/bin/bash

cd $(git rev-parse --show-toplevel)

cargo +nightly build -p isomorphic-client --target wasm32-unknown-unknown &&
  wasm-bindgen --no-typescript target/wasm32-unknown-unknown/debug/isomorphic_client.wasm --out-dir ./examples/isomorphic/client &&
./node_modules/webpack-cli/bin/cli.js --mode=development \
  ./examples/isomorphic/client/client-entry-point.js -o ./examples/isomorphic/client/bundle.js
