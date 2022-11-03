# Conditional Rendering

Sometimes you'll want to conditionally render some html. You can use an `Option`.

```rust
fn conditional_render() {
    let maybe_render: Option<VirtualNode> = make_view();

    html! {
        <div>
            <h1>Hello World</h1>
            { maybe_render }
        </div>
    }
}
```
