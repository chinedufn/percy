#!/bin/bash

cd $(git rev-parse --show-toplevel)

cargo build --all && \ # Make sure examples compile
cargo test --all && npm run test
