#!/bin/bash

cd $(dirname $0)

mkdir -p build/
mkdir -p dist/

cargo +nightly watch -w ./ -w ../app \
    -x 'build -p isomorphic-client --target wasm32-unknown-unknown' \
    -s 'wasm-bindgen --no-modules --no-typescript ../../../target/wasm32-unknown-unknown/debug/isomorphic_client.wasm --out-dir ./build' &
  ../../../node_modules/webpack-dev-server/bin/webpack-dev-server.js &
  wait
