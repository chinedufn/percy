#!/bin/bash

cd $(dirname $0)

cd ./client

./build-wasm.watch.sh &

systemfd --no-pid -s http::7878 -- OUTPUT_CSS=./build/app.css cargo +nightly watch -w ../server -w ../app -x 'run -p isomorphic-server' &

trap "exit" INT TERM
trap "kill 0" EXIT

wait
