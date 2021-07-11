Percy
=====

[![Actions Status](https://github.com/chinedufn/percy/workflows/percy-build-test/badge.svg)](https://github.com/chinedufn/percy/actions)

> Build frontend browser apps with Rust + WebAssembly. Supports server side rendering.

## The Percy Book

This README serves as a light introduction to Percy. Consult [The Percy Book] for a full walk through.

[The Percy Book]: https://chinedufn.github.io/percy/

## Stable Rust

Percy compiles on stable Rust, however there is one aspect of the `html-macro` that is different on stable at this time:

On nightly Rust you can create text nodes without quotes.

```rust
// Nightly Rust does not require quotes around text nodes.
html! { <div>My text nodes here </div> };
```

On stable Rust, quotation marks are required.

```rust
// Stable Rust requires quotes around text nodes.
html! { <div>{ "My text nodes here " }</div> };
```

This difference will go away once span locations are stabilized in the Rust compiler - [Rust tracking issue](https://github.com/rust-lang/rust/issues/54725).

## Getting Started

The best way to get up to speed is by checking out [The Percy Book](https://chinedufn.github.io/percy/), but here is a
very basic example to get your feet wet with.

For a full example of an isomorphic web app in Rust check out the [isomorphic example](examples/isomorphic).

### Quickstart - Getting your feet wet

Percy allows you to create applications that only have server side rendering, only client side rendering,
or both server and client side rendering.

Here's a quick-and-easy working example of client side rendering that you can try right now.

---

First, Create a new project using

```sh
cargo new client-side-web-app --lib
cd client-side-web-app
```

---

Add the following files to your project.

```sh
touch build.sh
touch index.html
```

---

Here's the end directory structure:

```sh
.
├── Cargo.toml
├── build.sh
├── index.html
└── src
    └── lib.rs
```

---

Now edit each file with the following contents:

```sh
# contents of build.sh

#!/bin/bash

cd "$(dirname "$0")"

mkdir -p public

CSS_FILE="$(pwd)/public/app.css"
OUTPUT_CSS=$CSS_FILE wasm-pack build --no-typescript --dev --target no-modules --out-dir ./public
cp index.html public/
```

---

```rust
// contents of src/lib.rs



use wasm_bindgen::prelude::*;
use web_sys;
use web_sys::MouseEvent;

use css_rs_macro::css;
use virtual_dom_rs::prelude::*;

#[wasm_bindgen]
struct App {
  dom_updater: DomUpdater
}

#[wasm_bindgen]
impl App {
    #[wasm_bindgen(constructor)]
    pub fn new () -> App {
        let start_view = html! { <div> Hello </div> };

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let body = document.body().unwrap();

        let mut dom_updater = DomUpdater::new_append_to_mount(start_view, &body);

        let greetings = "Hello, World!";

        let end_view = html! {
           // Use regular Rust comments within your html
           <div class="big blue">
              /* Interpolate values using braces */
              <strong>{ greetings }</strong>

              <button
                class=MY_COMPONENT_CSS
                onclick=|_event: MouseEvent| {
                   web_sys::console::log_1(&"Button Clicked!".into());
                }
              >
                // No need to wrap text in quotation marks (:
                Click me and check your console
              </button>
           </div>
        };

        dom_updater.update(end_view);

        App { dom_updater }
    }
}

static MY_COMPONENT_CSS: &'static str = css!{r#"
:host {
    font-size: 24px;
    font-weight: bold;
}
"#};

static _MORE_CSS: &'static str = css!{r#"
.big {
  font-size: 30px;
}

.blue {
  color: blue;
}
"#};
```

---

```toml
# contents of Cargo.toml

[package]
name = "client-side-web-app"
version = "0.1.0"
authors = ["Friends of Percy"]
edition = "2018"

[lib]
crate-type = ["cdylib"] # Don't forget this!

[dependencies]
wasm-bindgen = "0.2.37"
js-sys = "0.3.14"
virtual-dom-rs = "0.6"
css-rs-macro = "0.1"

[dependencies.web-sys]
version = "0.3"
features = [
    "Document",
    "MouseEvent",
    "Window",
    "console"
]
```

---

```html
<!-- contents of index.html -->
<!DOCTYPE html>
<html lang="en">
    <head>
        <meta charset="UTF-8">
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <link rel="stylesheet" type="text/css" href="app.css"/>
        <title>Client Side Demo</title>
    </head>
    <body style='margin: 0; padding: 0; width: 100%; height: 100%;'>
        <script src='/client_side_web_app.js'></script>
        <script>
            window.wasm_bindgen(`/client_side_web_app_bg.wasm`).then(() => {
                const { App } = window.wasm_bindgen
                new App()
            })
        </script>
    </body>
</html>
```

---

Now run

```sh
# Used to compile your Rust code to WebAssembly
cargo install wasm-pack

# Or any other static file server that supports the application/wasm mime type
npm install -g http-server

chmod +x ./build.sh
./build.sh

# Visit localhost:8080 in your browser
http-server ./public --open
```

And you should see the following:

![Client side example](./example.png)

Nice work!

## More Examples

- [Isomorphic web app](examples/isomorphic)

- [CSS in Rust](examples/css-in-rust)

- [Unit Testing View Components](examples/unit-testing-components)

- [Open an Issue or PR if you have an idea for a useful example!](https://github.com/chinedufn/percy/issues)

## API Documentation

- [virtual-dom-rs API docs](https://chinedufn.github.io/percy/api/virtual_dom_rs/macro.html.html)

- [html-macro API docs](https://chinedufn.github.io/percy/api/html_macro)

- [router-rs API docs](https://chinedufn.github.io/percy/api/router_rs)

- [css-rs-macro API docs](https://chinedufn.github.io/percy/api/css_rs_macro)

## Contributing

Always feel very free to open issues and PRs with any questions / thoughts that you have!

Even if it feels basic or simple - if there's a question on your mind that you can't quickly answer yourself then that's a failure
in the documentation.

Much more information on how to contribute to the codebase can be found in the [contributing section](https://chinedufn.github.io/percy/contributing/getting-started.html) of The Percy Book!

## To Test

To run all of the unit, integration and browser tests, [grab the dependencies then](https://chinedufn.github.io/percy/contributing/getting-started.html) :

```sh
./test.sh
```

## See Also

- [virtual-dom](https://github.com/Matt-Esch/virtual-dom) - a JavaScript virtual-dom implementation that I took inspiration from.

- [How to write your own Virtual DOM](https://medium.com/@deathmood/how-to-write-your-own-virtual-dom-ee74acc13060) - helped me better understand how a virtual-dom works.

- [Sheetify](https://github.com/stackcss/sheetify) inspired the css! macro

## License

MIT
