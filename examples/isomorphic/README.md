# isomorphic web app example

## Viewing a live version

[View the example online](https://percy-isomorphic.now.sh/)

## Running Locally

[Install rustup if you haven't already](https://rustup.rs/)

```
# Make sure you have Rust nightly
rustup install nightly
rustup update nightly
```

```sh
# Install wasm-bindgen
rustup target add wasm32-unknown-unknown
cargo +nightly install wasm-bindgen-cli
```

```sh
# Clone the Percy repository
git clone https://github.com/chinedufn/percy
cd percy
npm install
```

```sh
# Build the WebAssembly module and start the server
./examples/isomorphic/start.sh
# Now visit http://localhost:7878
```

## Structure

Percy powered isomorphic web applications use three crates in a cargo workspace.

An app crate, a client crate and a server crate.

### app crate

The app crate holds all of your application logic. It is responsible for generating
a virtual-dom given some application state. It also holds all of the methods for
updating application state.

### server crate

The server crate depends on your application crate. It initializes your application
with some initial state, renders your applications virtual-dom into an HTML string and then
serves that string to the client.

It also serializes the initial state into JSON and serves that to the client as well so
that the client can start off with the exact same state that the server initialized
the application with.

### client crate

The client crate is a `cdylib` that gets compiled to WebAssembly. This crate is a light
wrapper around your app crate, allowing you to run your code in the browser.

Seperating the web `client` logic from the `app` makes it easy for you to add other clients in the
future, such as an `electron` client.

## Changing the now.sh Dockerfile

We use a `Dockerfile` to deploy to `now.sh` (currently stored in the root directory but in the future we might move that)

To run it

```sh
docker build -t percy-isomorphic .
docker run -d -p 7878:7878 percy-isomorphic
# Visit localhost:7878 in your web browser
```

You can also do a release build outside docker, for that you will need `musl-tools` and the `x86_64-unknown-linux-musl` target.

```sh
sudo apt install musl-tools
rustup target add x86_64-unknown-linux-musl
```

```sh
./examples/isomorphic/build.release.sh
./examples/isomorphic/run.release.sh
```
