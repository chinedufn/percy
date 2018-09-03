#!/bin/bash

# cd to the root directory of this repository
cd $(dirname $0)
cd ../..

./examples/isomorphic/client/build-wasm.sh
cargo +nightly run -p isomorphic-server
