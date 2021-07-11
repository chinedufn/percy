# Introduction

> Note: The book is a work in progress. Some chapters are empty placeholders that will be filled in over time.

`Percy` a collection of libraries for building interactive frontend browser apps with Rust + WebAssembly.

Percy supports server side rendering out of the box.

`Percy` is not yet ready for production (unless you're incredibly brave), but if you're
interested in using it for real things you can [watch the development progress.](https://github.com/chinedufn/percy/watchers).

### What is Percy?

Percy is a toolkit geared towards building single page web apps entirely in Rust that can also be rendered at the server.

This allows you to build search engine friendly browser applications in Rust.

### A snippet

```rust


use virtual_dom_rs::prelude::*;

// Percy supports events, classes, attributes a virtual dom
// with diff/patch and everything else that you'd expect from
// a frontend toolkit.
//
// This, however, is just the most basic example of rendering
// some HTML on the server side.
fn main () {
  let some_component = html! {
    <div class="cool-component">Hello World</div>
  };

  let html_string = some_component.to_string();
  println!("{}", html_string);
}
```

### Roadmap

`Percy` is very young and going through the early stages of development. Our roadmap is
is mainly led by Real World Driven Development.

This means that we're using `Percy` to build a real, production web app and ironing out
the kinks and fixing the bugs as we go.

Once the tools have stabilized and we've settled into a clean structure for `Percy`
applications we'll publish a CLI for generating a production-grade starter project with
everything that you need to get up and running.

Check out the [Percy issue tracker](https://github.com/chinedufn/percy/issues) and
maybe open a couple of your own!

### Notable Features

`Percy` is still young, so the feature set is still growing and maturing. At the moment:

- An `html!` macro that generates a virtual dom that can can be rendered into a DOM element
on the frontend or a `String` on the backend.

- CSS in Rust - Optionally writing your CSS styles right next to your `html!` components instead
of in separate CSS/Sass/etc files.
