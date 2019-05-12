# On Visit Callback

You'll sometimes want to do something whenever you visit a route.

For example, you might want to download some data from an API whenever you
visit a route.

You can specify things like this using the `on_visit` attribute in the `#route(...)`
macro.

Doing this in the `#route(...)` macro makes it very clear what happens whenever
a route is visited.

```rust
// snippet: examples/isomorphic/app/src/lib.rs

{{#bookimport ../../../../examples/isomorphic/app/src/lib.rs@on-visit-example}}
```
