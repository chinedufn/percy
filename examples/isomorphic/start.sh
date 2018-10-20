#!/bin/bash

cd $(dirname $0)

cd ./client

./build-wasm.sh
OUTPUT_CSS="$(pwd)/build/app.css" cargo +nightly run -p isomorphic-server
