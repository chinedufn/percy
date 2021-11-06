# Real Elements and Nodes

You'll sometimes want to do something to the real DOM [Node] that gets created from your `VirtualNode`.

You can accomplish this with the `SpecialAttributes.on_create_elem` attribute function.

```rust
use virtual_node::wrap_closure;

let mut div: VirtualNode = html! {
<div>
    <span>This span should get replaced</span>
</div>
};

div.as_velement_mut()
    .unwrap()
    .special_attributes
    .on_create_elem = Some((
        "some-unique-key".into(),
        wrap_closure(move |elem: web_sys::Element| {
            elem.set_inner_html("Hello world");
        }),
    ));

let div: Element = div.create_dom_node().node.unchecked_into();

assert_eq!(div.inner_html(), "Hello world");
```

[Node]: https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Node.html
