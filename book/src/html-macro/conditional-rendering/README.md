# Conditional Rendering

Sometimes you'll want to conditionally render some html without an else statement. This isn't actually possible in Rust
because an if-else statement is an expression, this means that you can assign this to a variable as the types of the then
and else branches don't match.

Although this is quite a common practice in React and other web frameworks, and is supported in Percy.

You'll be able to include if statements without an else branch inside the html macro.

```rust
fn conditional_render() {
    html! {
        <div>
            <h1>Hello World</h1>
            {if should_show_child() {
                html! {
                    <p>This child component will only render if the condition evaluates to true</p>
                }
            }}
        </div>
    }
}
```
