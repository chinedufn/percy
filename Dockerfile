# Used to host the isomorphic web app example

FROM yasuyuky/rust-nightly-musl:latest as build

# Install wasm32-unknown-unknown-target
RUN rustup default nightly
RUN rustup target add wasm32-unknown-unknown \
  x86_64-unknown-linux-musl

# Node.js & npm
RUN curl -sL https://deb.nodesource.com/setup_10.x | bash -
RUN apt-get install -y nodejs 

# Install WASM bindgen CLI
RUN curl -OL https://github.com/rustwasm/wasm-bindgen/releases/download/0.2.23/wasm-bindgen-0.2.23-x86_64-unknown-linux-musl.tar.gz &&\
  tar xf wasm-bindgen-0.2.23-x86_64-unknown-linux-musl.tar.gz &&\
  rm wasm-bindgen-0.2.23-x86_64-unknown-linux-musl.tar.gz &&\
  chmod +x wasm-bindgen-0.2.23-x86_64-unknown-linux-musl/wasm-bindgen &&\
  mv wasm-bindgen-0.2.23-x86_64-unknown-linux-musl/wasm-bindgen /usr/local/bin/wasm-bindgen

WORKDIR /usr/src

COPY package.json package-lock.json ./

# Get npm package dependencies
RUN npm install

COPY . ./

WORKDIR /usr/src/examples/isomorphic

RUN ./build.release.sh

# This gets around the 100Mb limit by re-starting from a tiny image
# We tried `scratch` and `alpine:rust` but targeting them proved difficult so going the easy route.
FROM scratch

# At the moment our server expects the files to be in `/examples/isomorphic/client/{filename}` so we copy the examples dir
COPY --from=build /usr/src/target/x86_64-unknown-linux-musl/release/isomorphic-server /
COPY --from=build /usr/src/examples/isomorphic/client/dist /dist

EXPOSE 7878/tcp

WORKDIR /
ENTRYPOINT ["/isomorphic-server"]
