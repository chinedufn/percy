#!/bin/bash

cd $(git rev-parse --show-toplevel)

./examples/isomorphic/client/build-wasm.sh
cargo +nightly run -p isomorphic-server
