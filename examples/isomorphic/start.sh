#!/bin/bash

cd $(dirname $0)

cd ./client

./build-wasm.sh
cargo +nightly run -p isomorphic-server
