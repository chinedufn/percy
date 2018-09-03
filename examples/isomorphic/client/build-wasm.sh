#!/bin/bash

# cd to the root directory of this repository
cd $(dirname $0)
cd ../..

cargo +nightly build -p isomorphic-client --target wasm32-unknown-unknown &&
  wasm-bindgen --no-typescript target/wasm32-unknown-unknown/debug/isomorphic_client.wasm --out-dir ./examples/isomorphic/client &&
./node_modules/webpack-cli/bin/cli.js --mode=development \
  ./examples/isomorphic/client/client-entry-point.js -o ./examples/isomorphic/client/bundle.js
