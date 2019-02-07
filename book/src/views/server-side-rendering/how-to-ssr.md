# How to SSR

In the most simple case, server side rendering in `Percy` boils down to
rendering your virtual DOM to a `String` and responding to a client with
that `String`.

```rust
use virtual_dom_rs::prelude::*;
use std::cell::Cell;

fn main () {
  let count_cell = Cell::new(5);

  let app = html! {
    <div id="app">
      <button onclick=|_ev| { *count+= 1; }>
        Hello world
      </button>
    </div>
  };


  let html_to_serve = app.to_string();
  // <div id="app"><button>Hello world</button></div>

  // .. server string to client (http response) ...
}
```

## Hydrating initial state

You'll usually want your views to be rendered based on some application state. So, typically, your server will

1. Receive a request from the client
2. Set the initial application state based on the request
3. Render the application using the initial state
4. Reply with the initial HTML and the initial state
5. Client takes over rendering, starting from the initial state.

To illustrate we'll take a look at an excerpt from a more realistic server side rendering example.

Afterwards you can check out the full example at [examples/isormorphic](https://github.com/chinedufn/percy/tree/master/examples/isomorphic).

---

A more realistic server side rendering implementation would look like the following:

```html
{{#include ../../../../examples/isomorphic/server/src/index.html}}
```

```rust
// examples/isormorphic/server/src/server.rs
// Check out the full application in /examples/isormorphic directory
{{#include ../../../../examples/isomorphic/server/src/server.rs}}
```

And then the client would use `serde` to deserialize the `initialState`
into a State struct and begin rendering using that State.
