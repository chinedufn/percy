# Used to host the isomorphic web app example

FROM rust:latest as build

WORKDIR /usr/src

COPY . .

RUN rustup update nightly
RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown

# Install WASM bindgen CLI
RUN curl -OL https://github.com/rustwasm/wasm-bindgen/releases/download/0.2.19/wasm-bindgen-0.2.19-x86_64-unknown-linux-musl.tar.gz
RUN tar xf wasm-bindgen-0.2.19-x86_64-unknown-linux-musl.tar.gz
RUN rm wasm-bindgen-0.2.19-x86_64-unknown-linux-musl.tar.gz
RUN chmod +x wasm-bindgen-0.2.19-x86_64-unknown-linux-musl/wasm-bindgen
RUN mv wasm-bindgen-0.2.19-x86_64-unknown-linux-musl/wasm-bindgen /usr/local/bin/wasm-bindgen

# Node.js & npm
RUN curl -sL https://deb.nodesource.com/setup_9.x | bash -
RUN apt-get install -y nodejs

# Build tools for making npm install work
RUN apt-get install -y build-essential
RUN apt-get install -y libssl-dev
RUN apt-get install -y pkg-config

RUN npm install

# Build wasm target
RUN cargo +nightly build -p isomorphic-client --release --target wasm32-unknown-unknown
RUN wasm-bindgen --no-typescript target/wasm32-unknown-unknown/release/isomorphic_client.wasm --out-dir ./examples/isomorphic/client

# Build WASM module
# TODO: --mode=production . Need to make sure it works locally. If it doesn't try disabling mangling
RUN ./node_modules/webpack-cli/bin/cli.js --mode=development ./examples/isomorphic/client/client-entry-point.js -o ./examples/isomorphic/client/bundle.js

RUN cargo +nightly build -p isomorphic-server --release

# Diagnostics to see if things are working..
RUN pwd
RUN ls target
RUN ls target/release
RUN ls target/release/isomorphic-server

# This gets around the 100Mb limit by ditching our `target` directory
FROM alpine:latest

# At the moment our server expects the files to be in `/examples/isomorphic/client/{filename}` so we copy the examples dir
# In the future we might conditionally just `include_bytes!` for production builds instead of
# reading the bundle.js file from disk. We read it from disk atm to avoid recompilation when it changes.
COPY --from=build /usr/src/target/release/isomorphic-server /
COPY --from=build  /usr/src/examples /examples


RUN ls
RUN pwd
RUN ls /

RUN chmod +x /isomorphic-server
CMD ["/isomorphic-server"]
