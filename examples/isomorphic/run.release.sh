#!/bin/bash

cd $(dirname $0)

cd ./client

../../../target/x86_64-unknown-linux-musl/release/isomorphic-server
