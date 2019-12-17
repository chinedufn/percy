# Custom Components

Percy's `html!` macro supports custom components.

You can create a component by implementing the `View` trait.

Here is an example:

```rust
fn page() -> VirtualNode {
    html! {
        <div>
            <ChildView count={0}/>
        </div>
    }
}

struct ChildView {
    count: u8,
}

impl View for ChildView {
    fn render(&self) -> VirtualNode {
        html! {
            <div>
                Count is {format!("{}", self.count)}
            </div>
        }
    }
}
```
