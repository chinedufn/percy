# Used to host the isomorphic web app example

FROM rust:latest as build

WORKDIR /usr/src

COPY . .

# Install compilation targets
RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown
RUN rustup target add x86_64-unknown-linux-musl

# Install cross for cross compilation to alpine linux
RUN curl -OL https://github.com/rust-embedded/cross/releases/download/v0.1.14/cross-v0.1.14-x86_64-unknown-linux-gnu.tar.gz
RUN tar xf cross-v0.1.14-x86_64-unknown-linux-gnu.tar.gz
RUN rm cross-v0.1.14-x86_64-unknown-linux-gnu.tar.gz
# Make sure it doesn't stay in the developers file system if someone builds this locally
RUN mv cross /root/

# Build example isomorphic server binary
RUN /root/cross build -p isomorphic-server --release --target x86_64-unknown-linux-musl
RUN ls /usr/src/target/release
RUN ls /usr/src/target/x86_64-unknown-linux-musl

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

# Ge npm package dependencies
RUN npm install

# Build wasm target
RUN cargo build -p isomorphic-client --release --target wasm32-unknown-unknown
RUN wasm-bindgen --no-typescript target/wasm32-unknown-unknown/release/isomorphic_client.wasm --out-dir ./examples/isomorphic/client

# Build WASM module
# TODO: --mode=production . Need to make sure it works locally. If it doesn't try disabling UglifyJS mangling
RUN ./node_modules/webpack-cli/bin/cli.js --mode=development ./examples/isomorphic/client/client-entry-point.js -o ./examples/isomorphic/client/bundle.js

# This gets around the 100Mb limit by re-starting from a tiny image
FROM scratch

# At the moment our server expects the files to be in `/examples/isomorphic/client/{filename}` so we copy the examples dir
COPY --from=build /usr/src/target/x86_64-unknown-linux-musl/release/isomorphic-server /
COPY --from=build  /usr/src/examples /examples

CMD ["/isomorphic-server"]
