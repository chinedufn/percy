#!/bin/bash

set -e

cd $(git rev-parse --show-toplevel)

cargo check --all
cargo test --all

# Ensure that `crates/virtual-dom` can compile without web-sys/js-sys/wasm-bindgen.
cargo check -p virtual-node --no-default-features

# Run the browser tests using `wasm-bindgen-test-runner` (see `.cargo/config.toml`).
# Defaults to Chrome. Set `GECKODRIVER` or `SAFARIDRIVER` to use Firefox or Safari instead.
if [ -z "${CHROMEDRIVER}${GECKODRIVER}${SAFARIDRIVER}" ]; then
  export CHROMEDRIVER=chromedriver
fi
cargo test -p percy-dom --target wasm32-unknown-unknown
