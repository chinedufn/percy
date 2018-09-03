#!/bin/bash

# cd to the root directory of this repository
cd $(dirname $0)
cd ../..

exit 0

./examples/isomorphic/client/build-wasm.sh
cargo +nightly run -p isomorphic-server
