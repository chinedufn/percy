# Router

`router-rs` provides functionality that helps you render different views when your users' visit different routes.

Let's take a look:

```rust
// Imported from crates/router-rs-macro-test/src/lib.rs

{{#include ../../../crates/router-rs-macro-test/src/lib.rs:116:137}}
```

> NOTE: that we used `VirtualNode::new` in this snippet but you'd
typically use the `html!` macro to generate a `VirtualNode`
