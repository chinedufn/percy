#!/bin/bash

cd $(dirname $0)

set -e

mkdir -p build/
mkdir -p dist/

wasm-pack build --dev --target web --no-typescript --out-dir ./build
