# Virtual DOM

At the heart of the `Percy` toolkit is `virtual-dom-rs`, a crate that provides a virtual dom
implementation that allows you to write functional front-end applications.

This same `virtual-dom-rs` also works on the backend by rendering to a String instead of a DOM element.
This ability to render on the backend is commonly referred to as server side rendering.

```rust
use virtual_dom_rs::prelude::*;

// The most basic example of rendering to a String
fn main () {
  let component = html! { <div id="my-id"> Hello world </div> };
  println!("{}", component);
  // <div id="my-id">Hello world</div>
}
```
