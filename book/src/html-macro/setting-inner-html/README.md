# Setting Inner HTML

You'll sometimes want to use a string of HTML in order to set the child nodes for an element.

For example, if you're creating a tool tip component you might want to be able to support setting tool tips using
arbitrary HTML such as `"Hello <strong>World!</strong>"`:

You can use the `SpecialAttributes.dangerous_inner_html` attribute to set inner html.

Note that it is called `dangerous` because it can potentially expose your application to [cross-site scripting][XSS] attacks if your application
trusts arbitrary un-escaped HTML strings.

```rust

let tooltip_contents = "<span>hi</span>";

let mut div: VirtualNode = html! {
<div></div>
};
div.as_velement_mut()
    .unwrap()
    .special_attributes
    .dangerous_inner_html = Some(tooltip_contents.to_string());

let div: Element = div.create_dom_node().node.unchecked_into();

assert_eq!(div.inner_html(), "<span>hi</span>");
```

[XSS]: https://en.wikipedia.org/wiki/Cross-site_scripting
