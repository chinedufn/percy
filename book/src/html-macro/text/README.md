# Working with Text

Rather than needing you to wrap your text in `"` quotation marks,
the `html-macro` will work with raw unquoted text.

```rust
fn main () {
    let interpolated_var = "interpolate text variables.";

    let example = html! {
       <div>
            Text can be typed directly into your HTML.
            <div>Or you can also {interpolated_var}</div>
       </div>
    };
}
```

You should always get the same spacing (or lack there of) between text and/or elements as you would
if you were working in a regular old `.html` file.

When it comes to interpolated variables, we base spacing on the spacing outside of the brackets.

```rust
fn main () {
    let text = "hello";

    html! { <div>{ hello }</div> }; // <div>hello</div>
    html! { <div>{hello}</div> }; // <div>hello</div>

    html! { <div> { hello } </div> }; // <div> hello </div>
    html! { <div> {hello} </div> }; // <div> hello </div>

    html! { <div>{hello} </div> }; // <div>hello </div>
    html! { <div> {hello}</div> }; // <div> hello</div>
}
```

## More Examples

Here are a bunch of examples showing you what happens when you try and mix
text nodes / variables / elements.

```rust
// Imported into book from crates/html-macro-test/src/text.rs

{{ #include ../../../../crates/html-macro-test/src/text.rs }}
```
