#!/bin/bash

cd $(dirname $0)

cd ./client

./build-wasm.dev.sh &
systemfd --no-pid -s http::7878 -- cargo +nightly watch -w ../ -x 'run -p isomorphic-server' &
wait
