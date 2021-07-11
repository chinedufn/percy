# Real Elements and Nodes

You'll sometimes want to do something to the real DOM [Node] that gets created from your `VirtualNode`.

You can accomplish this with the `on_create_elem` function.

```rust
{{#bookimport ../../../../../crates/percy-dom/tests/create_element.rs@on-create-elem}}
```

[Node]: https://rustwasm.github.io/wasm-bindgen/api/web_sys/struct.Node.html
