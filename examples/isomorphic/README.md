# isomorphic web app example

To run locally:

```sh
./start.sh
```

---

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

---

While the `client` crate needs to be separate since it's a [cdylib](https://doc.rust-lang.org/reference/linkage.html#linkage) that gets compiled to WebAssembly, you
could combine the app and server crate if you wanted. I'll have to try that and see which feels better.

This might look like

1. app crate (server module AND a pub app module)
2. webassembly crate (`cdylib` that depends on your `app` crate pulls in your app module)

Not sure which I like better on paper... need to try both... Please open an issue if you have opinions!
