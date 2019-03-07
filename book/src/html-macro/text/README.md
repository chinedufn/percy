# Working with Text

One of the most popular types of nodes in the DOM is the [Text] node, and the `html! macro
focuses heavily on making them as easy to create as possible.

You can just type unquoted text into the `html!` macro and neighboring text will get combined into a single `Text` node, much
like the way that web browsers handle text from html documents.

```rust
fn main () {
    let interpolated_text = "interpolate text variables.";

    let example = html! {
       <div>
            Text can be typed directly into your HTML.
            <div>Or you can also {interpolated_text}</div>
       </div>
    };
}
```

## Preserving Space Between Blocks

You should always get the same spacing (or lack there of) between text and other elements as you would
if you were working in a regular old `.html` file.

We'll preserve newline characters so that `white-space: pre-wrap` etc will work as expected.

When it comes to interpolated variables, we base spacing on the spacing outside of the braces, not the
inside.

Let's illustrate:

```rust
fn main () {
    let text = "hello";

    html! { <div>{ hello }</div> }; // <div>hello</div>
    html! { <div>{hello}</div> }; // <div>hello</div>

    html! { <div> { hello } </div> }; // <div> hello </div>
    html! { <div> {hello} </div> }; // <div> hello </div>

    html! { <div>{hello} </div> }; // <div>hello </div>
    html! { <div>   {hello}</div> }; // <div>   hello</div>
}
```

[Text]: https://developer.mozilla.org/en-US/docs/Web/API/Text
