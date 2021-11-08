# On Remove Element

The `on_remove_elem` special attribute allows you to register a function that will be called
when the element is removed from the DOM.

```rust
let mut div: VirtualNode = html! {
    <div></div>
};

div.as_velement_mut()
    .unwrap()
    .special_attributes
    .set_on_remove_element(
       "some-key",
        move |_elem: web_sys::Element| {
          // ...
        },
    ));
```

## Macro shorthand

You can also use the `html!` macro to set the `on_remove_element` function.

```rust
let _ = html! {
  <div
    key="some-key"
	on_remove_element = move |_element: web_sys::Element| {
	  // ...
	}
  >
    Before
  </div>
}
```
