# isomorphic web app example

*TODO: Find a service where we can host a live version of the demo for free or cheaply.*

## Running Locally

[Install rustup if you haven't already](https://rustup.rs/)

```sh
# Install wasm-pack
cargo install wasm-pack

# Clone the Percy repository
git clone https://github.com/chinedufn/percy
cd percy

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
