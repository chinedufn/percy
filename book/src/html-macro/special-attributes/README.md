# Special Attributes

Some attributes do not merely set or remove the corresponding HTML attribute of the same name.

## `checked` Attribute

Specifying `checked` causes `percy` to render the checkbox with the specified checkedness, INSTEAD of setting the default checkedness.

The `checked` HTML attribute specifies the [default checkedness of an input element](https://html.spec.whatwg.org/multipage/input.html#attr-input-checked). It does not determine the checkedness of the checkbox directly.

From the link above:

> The checked content attribute is a boolean attribute that gives the default checkedness of the input element. When the checked content attribute is added, if the control does not have dirty checkedness, the user agent must set the checkedness of the element to true; when the checked content attribute is removed, if the control does not have dirty checkedness, the user agent must set the checkedness of the element to false.

A developer is likely to use `html!`'s `checked` attribute and expect the value they specify to be the value that is rendered. Setting the `checked` HTML attribute alone does not achieve this.

To avoid this, `html!`'s `checked` specifies the rendered checkedness directly, using `set_checked` behind the scenes.

```rust
html! { <input type="checkbox" checked=true> };
```

It's still possible to use `elem.set_attribute("checked", "")` and `elem.remove_attribute("checked")` to configure the default checkedness.

```rust
let vnode = html! {<input type="checkbox">};

let mut events = VirtualEvents::new();
let (input_node, enode) = vnode.create_dom_node(&mut events);
events.set_root(enode);

// Sets the default checkedness to true by setting the `checked` attribute.
let input_elem = input_node.dyn_ref::<HtmlInputElement>().unwrap();
input_elem.set_attribute("checked", "").unwrap();
```

## `value` Attribute

Specifying `value` causes `percy` to render the the input's value as specified, as well as setting the default value.

Similar to `checked`, the `value` HTML attribute [specifies the default value of an input element](https://html.spec.whatwg.org/multipage/input.html#attr-input-value). It does not determine the value of the element directly.

From the link above:

> The value content attribute gives the default value of the input element. When the value content attribute is added, set, or removed, if the control's dirty value flag is false, the user agent must set the value of the element to the value of the value content attribute, if there is one, or the empty string otherwise, and then run the current value sanitization algorithm, if one is defined.

A developer is likely to use `html!`'s `value` attribute and expect the value they specify to be the value that is rendered. Setting the `value` HTML attribute alone does not achieve this.

To avoid this, `html!`'s `value` specifies the rendered value directly, using `set_value` behind the scenes.

```rust
html! { <input value="hello!"> };
```

Note that **unlike the `checked` attribute**, `percy` applies the specified value by setting the `value` attribute as well as using `set_value`.
