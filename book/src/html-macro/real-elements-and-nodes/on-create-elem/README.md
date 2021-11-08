# On Create Element

The `on_create_elem` special attribute allows you to register a function that will be called
when the element is first created.

```rust
let mut div: VirtualNode = html! {
<div>
    <span>This span should get replaced</span>
</div>
};

div.as_velement_mut()
    .unwrap()
    .special_attributes
    .set_on_create_element(
        "some-key",
        move |elem: web_sys::Element| {
            elem.set_inner_html("Hello world");
        },
    ));

let div: Element = div.create_dom_node().node.unchecked_into();

assert_eq!(div.inner_html(), "Hello world");
```

## Macro shorthand

You can also use the `html!` macro to set the `on_create_element` function.

```rust
let _ = html! {
  <div
    key="some-key"
	on_create_element = move |element: web_sys::Element| {
	    element.set_inner_html("After");
	}
  >
    Before
  </div>
}
```
