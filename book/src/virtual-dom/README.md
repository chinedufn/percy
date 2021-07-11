# Virtual DOM

At the heart of the `Percy` toolkit is `percy-vdom`, a crate that provides a virtual dom
implementation that allows you to write functional front-end applications.

This same `percy-vdom` also works on the backend by rendering to a String instead of a DOM element.
This ability to render on the backend is commonly referred to as server side rendering.

```rust
use percy_vdom::prelude::*;

// The most basic example of rendering to a String
fn main () {
  let component = html! { <div id="my-id"> Hello world </div> };
  println!("{}", component);
  // <div id="my-id">Hello world</div>
}
```
