# Lists

## Keys

When elements in a list are keyed the diffing and patching functions
will use the nodes' keys to know whether an element was removed or simply
moved.

This leads to fewer interactions with the real-DOM when modifying lists,
as well as being able to preserve child elements when keyed elements are moved
around in the list.

This preservation is useful when you have an element that has children
that aren't managed by percy-dom.

Using keys in lists is recommended, but not required.

Here's an example of using the key attribute:

```rust
let items = ["a", "b", "c"];
let items: Vec<VirtualNode> = items
    .into_iter()
    .map(|key| {
        html! { <div key={key}>Div with key {key}</div> }
    })
    .collect();

let node = html! {
  <div>
    { items }
  </div>
}
```
