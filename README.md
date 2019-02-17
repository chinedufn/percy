Percy
=====

[![Build status](https://circleci.com/gh/chinedufn/percy.svg?style=shield&circle-token=:circle-token)](https://circleci.com/gh/chinedufn/percy)

> A modular toolkit for building [isomorphic web apps][isomorphic-web-apps] with Rust + WebAssembly

## The Percy Book

[The Percy book](https://chinedufn.github.io/percy/)

## Live Demo

[View the isomorphic web app example live](https://percy-isomorphic.now.sh/?init=42)

---

## What is an isomorphic web app?
[isomorphic-web-apps]: #isomorphic-web-apps

An isomorphic web application allows the same application code (in our case Rust code) to be run on both the server-side and the client-side (usually a web browser).

In a browser our application renders to an `HtmlElement`, and on the server our application renders to a `String`.

## API Documentation

- [virtual-dom-rs API docs](https://chinedufn.github.io/percy/api/virtual_dom_rs/macro.html.html)

- [css-rs-macro API docs](https://chinedufn.github.io/percy/api/css_rs_macro)

## Getting Started

For an example of an isomorphic web app in Rust check out the [isomorphic example](examples/isomorphic) or
view [the isomorphic web app example live.](https://percy-isomorphic.now.sh/)

For more on the `html!` macro see [html macro](virtual-dom-rs/src/html_macro.rs)

```rust
#![feature(proc_macro_hygiene)]

use virtual_dom_rs::prelude::*;
use css_rs_macro::css;

fn main () {
    let count = Rc::new(Cell::new(0));

    let count_clone = Rc::clone(count);

    let html = html! {
      <div id="hello-world" class=SOME_COMPONENT_CSS>
        <span>Hey there!</span>
        <button
          onclick=|_event: web_sys::MouseEvent| { count_clone.set(count_clone.get() + 1); },
          // CSS in Rust isn't required. You can use regular old
          /* classes just fine! */
          class="btn-bs4 btn-bs4-success"
        >
          Click Me!
        </button>
      </div>
    };

    // Used for server side rendering
    println!("{}", html.to_string());

    // Check out the DomUpdater for client side rendering
}

static SOME_COMPONENT_CSS: &'static str = css! {"
:host {
    font-size: 30px;
    font-weight: bold;
}

:host > span {
    color: blue;
}
"};
```

## Examples

- [Isomorphic web app](examples/isomorphic)

- [CSS in Rust](examples/css-in-rust)

- [Unit Testing View Components](examples/unit-testing-components)

- [Open an Issue or PR if you have an idea for a useful example!](https://github.com/chinedufn/percy/issues)

## Contributing

Please open issues / PRs explaining your intended use case and let's see if we should or shouldn't make `percy` support it!

Also feel free to open issues and PRs with any questions / thoughts that you have!

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
