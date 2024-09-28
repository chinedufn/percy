# Special Attributes

Some attributes do not merely set or remove the corresponding HTML attribute of the same name.

## `checked` Attribute

According to the [HTML spec](https://html.spec.whatwg.org/multipage/input.html#attr-input-checked), the `checked` HTML attribute only controls the default checkedness.
Changing the `checked` HTML attribute may not cause the checkbox's checkedness to change.

By contrast: specifying `html! { <input checked={bool} /> }` causes `percy` to always render the checkbox with the specified checkedness.
- If the VDOM is updated from `html! { <input checked=true /> }` to `html { <input checked=false /> }`, the input element's checkedness will definitely change.
- If the VDOM is updated from `html! { <input checked=true /> }` to `html { <input checked=true /> }`, the input element's checkedness will be reverted to `true` even if the user interacted with the checkbox in between.

`percy` updates both
- the `checked` attribute (default checkedness, reflected in HTML) and,
- the `checked` property (current checkedness, not reflected in HTML).

This behavior is more desirable because `percy` developers are accustomed to declaratively controlling the DOM and rendered HTML.

## `value` Attribute

According to the [HTML spec](https://html.spec.whatwg.org/multipage/input.html#attr-input-value), the `value` HTML attribute only controls the default value.
Changing the `value` HTML attribute may not cause the input element's value to change.

By contrast: specifying `html! { <input value="..." /> }` causes `percy` to always render the input element with the specified value.
- If the VDOM is updated from `html! { <input value="hello" /> }` to `html { <input value="goodbye" /> }`, the input element's value will definitely change.
- If the VDOM is updated from `html! { <input value="hello" /> }` to `html { <input value="hello" /> }`, the input element's value will be reverted to `"hello"` even if the user interacted with the input element in between.

`percy` updates both
- the `value` attribute (default value, reflected in HTML) and,
- the `value` property (current value, not reflected in HTML).

This behavior is more desirable because `percy` developers are accustomed to declaratively controlling the DOM and rendered HTML.
