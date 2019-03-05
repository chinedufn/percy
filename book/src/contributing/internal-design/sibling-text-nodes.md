# Sibling text nodes

If you render two text nodes next to them the browser will see them as just
one text node.

For example, when you have a component that looks like this:

```rust
use virtual_dom_rs::prelude::*;

let world = "world";

let sibling_text_nodes = html! { <div> hello {world} </div> };
```

A browser will end up with something like this:

```html
 <div>Hello World</div>
```

The `textContent` of the div in the browser is now "Hello World".

If we did not work around this behavior we wouldn't be able  to patch the DOM when two text nodes are next to each other.
We'd have no way of knowing how to find the original, individual strings that we wanted to render.

To get around this here's what we actually end up rendering:

```html
<div>Hello <!--ptns-->World</div>
```

Note the new `<!--ptns-->` comment node. Here's what `virtual_dom_rs`'s `createElement()` method ended up doing:

1. Saw the "Hello" virtual text and appended a real Text node into the real DOM `<div>`
2. Saw the "World" virtual text and saw that the previous element was also a virtual text node
3. Appended a `<!--ptns>` real comment element into the `<div>`
4. Appended a real "World" Text node into the `<div>`

If we later wanted to patch the DOM with a new component

```
let different_text = "there";
let sibling_text_nodes = html! { <div> hello {different_text} } </div> };
```

Our `virtual_dom_rs` patch function would be able to find the old "World" text node since we've ensured that it
did not get merged in with any other text nodes.
