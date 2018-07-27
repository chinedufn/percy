Percy [![Build Status](https://travis-ci.org/chinedufn/percy.svg?branch=master)](https://travis-ci.org/chinedufn/percy) [![docs](https://docs.rs/percy/badge.svg)](https://docs.rs/percy)
===============

> A modular toolkit for building isomorphic web apps with Rust + WebAssembly

_The tools:_

- virtual-dom-rs

- inline-stylesheets <- WIP

## Initial Background / Motivation

I started using Rust in January 2018 and quickly got to the stage of "I REALLY want to use this for everything, even if it isn't the best tool for the job."

I need to make a website for a game that I'm working on, but the Rust ecosystem for frontend web apps with server side rendering is still very immature.

So I started working on a standalone virtual-dom implementation that could render to an HTML string on the server side and to a DOM element in the browser.

But then I realized that I wanted something similar to [sheetify](https://github.com/stackcss/sheetify).. And probably a couple other base web dev primitives too..

So I decided to make a cargo workspace with the tools that I needed to build isomorphic web apps in Rust. And here we are!

## Getting Started

For an example of an isomorphic web app in Rust check out the [isomorphic example](examples/isomorphic)

For more on the `html!` macro see [html macro](virtual-dom-rs/src/html_macro.rs)

```rust
#[macro_use]
extern crate virtual_dom_rs;

fn main () {
  let count = Rc::new(Cell::new(0));

  let count_clone = Rc::clone(count);

  let html = html! {
    <div id="hello-world",>
      <button
        !onclick=|| { count_clone.set(count_clone.get() + 1); },
      >
        Click Me!
      </button>
      { "This becomes a text node" }
    </div>
  };

  println!("{}", html.to_string());
}
```

## Running the example isomorphic web app locally

Install the WASM compiler, if you haven't already. See [Setup WASM target](https://www.hellorust.com/setup/wasm-target/) for more information:

```sh
rustup target add wasm32-unknown-unknown --toolchain nightly
```

Then clone the source and run:

```sh
git clone https://github.com/chinedufn/percy
cd percy
./examples/isomorphic/start.sh
```

Now visit `http://127.0.0.1:7878` !

## Contributing

Please open issues / PRs explaining your intended use case and let's see if we should or shouldn't make `percy` support it!

Also feel free to open issues and PRs with any questions / thoughts that you have!

## To test

To run all of the Rust unit tests, Rust integration tests, and Node.js + WebAssembly tests run:

```sh
npm install
./test.sh
```

You'll need to be on Node.js 10.5+

## See Also

- [virtual-dom](https://github.com/Matt-Esch/virtual-dom) - a JavaScript virtual-dom implementation that I took inspiration from.

- [How to write your own Virtual DOM](https://medium.com/@deathmood/how-to-write-your-own-virtual-dom-ee74acc13060) - helped me better understand how a virtual-dom works.

- [Sheetify](https://github.com/stackcss/sheetify) inspired the css! macro

## License

MIT
