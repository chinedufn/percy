#!/bin/bash

cd $(dirname $0)

mkdir -p build/
mkdir -p dist/

wasm-pack build --target web --no-typescript --out-dir ./build
