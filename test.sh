#!/bin/bash

cd $(git rev-parse --show-toplevel)

cargo test --all && npm run test
