#!/bin/bash

cd $(dirname $0)

rm -rf dist/
mkdir -p dist/

wasm-pack build -p isomorphic-client --out-dir ./dist --release
