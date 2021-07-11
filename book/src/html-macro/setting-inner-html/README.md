# Setting Inner HTML

You'll sometimes want to use a string of HTML in order to set the child nodes for an element.

For example, if you're creating a tooltip component you might want to be able to support setting tooltips as such:

<div data-tip="Hello <strong>World!</strong>"></div>

You can use the `unsafe_inner_html` attribute for this purpose.

Note that it is called `unsafe` because it can poentially expose your application to [cross-site scripting][XSS] attacks if your application
trusts arbitrary un-escaped HTML strings that are provided by users.

```rust
{{#bookimport ../../../../crates/percy-vdom/tests/create_element.rs@inner-html}}
```

[XSS]: https://en.wikipedia.org/wiki/Cross-site_scripting
