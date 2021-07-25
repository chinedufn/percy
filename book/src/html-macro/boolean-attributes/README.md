# Boolean Attributes

Boolean attributes such as `disabled` and `checked` can be added by assigning a bool as their value.

Both variables and expressions can be used.

Here are a few examples:

```rust
let is_disabled = true;
let video_duration = 500;

html! {
    <video autoplay=false></video>

    <button disabled=is_disabled>Disabled Button</disabled>

    <video controls={video_duration > 100}></video>
}
```
