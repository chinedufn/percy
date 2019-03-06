#![feature(proc_macro_hygiene)]
#![cfg(test)]

use html_macro::html;
use std::collections::HashMap;
use virtual_node::{IterableNodes, VElement, VirtualNode};

struct HtmlMacroTest<'a> {
    desc: &'a str,
    generated: VirtualNode,
    expected: VirtualNode,
}

impl<'a> HtmlMacroTest<'a> {
    /// Ensure that the generated and the expected virtual node are equal.
    fn test(self) {
        assert_eq!(self.expected, self.generated, "{}", self.desc);
    }
}

#[test]
fn empty_div() {
    HtmlMacroTest {
        desc: "Empty div",
        generated: html! { <div></div> },
        expected: VirtualNode::element("div"),
    }
    .test();
}

#[test]
fn one_attr() {
    let mut attrs = HashMap::new();
    attrs.insert("id".to_string(), "hello-world".to_string());
    let mut expected = VElement::new("div");
    expected.attrs = attrs;

    HtmlMacroTest {
        desc: "One attribute",
        generated: html! { <div id="hello-world"></div> },
        expected: expected.into(),
    }
    .test();
}

#[test]
fn event() {
    HtmlMacroTest {
        desc: "Events are ignored in non wasm-32 targets",
        generated: html! {
            <div onclick=|_: u8|{}></div>
        },
        expected: html! {<div></div>},
    }
    .test();
}

#[test]
fn child_node() {
    let mut expected = VElement::new("div");
    expected.children = vec![VirtualNode::element("span")];

    HtmlMacroTest {
        desc: "Child node",
        generated: html! { <div><span></span></div> },
        expected: expected.into(),
    }
    .test();
}

#[test]
fn sibling_child_nodes() {
    let mut expected = VElement::new("div");
    expected.children = vec![VirtualNode::element("span"), VirtualNode::element("b")];

    HtmlMacroTest {
        desc: "Sibling child nodes",
        generated: html! { <div><span></span><b></b></div> },
        expected: expected.into(),
    }
    .test();
}

#[test]
fn three_nodes_deep() {
    let mut child = VElement::new("span");
    child.children = vec![VirtualNode::element("b")];

    let mut expected = VElement::new("div");
    expected.children = vec![child.into()];

    HtmlMacroTest {
        desc: "Nested 3 nodes deep",
        generated: html! { <div><span><b></b></span></div> },
        expected: expected.into(),
    }
    .test()
}

#[test]
fn sibling_text_nodes() {
    let mut expected = VElement::new("div");
    expected.children = vec![VirtualNode::text("This is a text node")];

    HtmlMacroTest {
        desc: "Nested text node",
        generated: html! { <div>This is a text node</div> },
        expected: expected.into(),
    }
    .test();
}

#[test]
fn nested_macro() {
    let child_2 = html! { <b></b> };

    let mut expected = VElement::new("div");
    expected.children = vec![
        VirtualNode::text(" "),
        VirtualNode::element("span"),
        VirtualNode::text(" "),
        VirtualNode::element("b"),
        VirtualNode::text(" "),
    ];

    HtmlMacroTest {
        desc: "Nested macros",
        generated: html! {
          <div>
            { html! { <span></span> } }
            { child_2 }
          </div>
        },
        expected: expected.into(),
    }
    .test();
}

#[test]
fn block_root() {
    let em = html! { <em></em> };

    let expected = VirtualNode::element("em");

    HtmlMacroTest {
        desc: "Block root node",
        generated: html! {
            { em }
        },
        expected,
    }
    .test();
}

#[test]
fn text_next_to_block() {
    let child = html! { <ul></ul> };

    let mut expected = VElement::new("div");
    expected.children = vec![
        VirtualNode::text(" A bit of text "),
        VirtualNode::element("ul"),
        VirtualNode::text(" "),
    ];

    HtmlMacroTest {
        desc: "Text node next to a block",
        generated: html! {
          <div>
            A bit of text
            { child }
          </div>
        },
        expected: expected.into(),
    }
    .test();
}

/// Ensure that we maintain the correct spacing around punctuation tokens
#[test]
fn punctuation_token() {
    let text = "Hello, World";

    HtmlMacroTest {
        desc: "Punctuation token spacing",
        generated: html! { Hello, World },
        expected: VirtualNode::text(text),
    }
    .test()
}

#[test]
fn vec_of_nodes() {
    let children = vec![html! { <div> </div>}, html! { <strong> </strong>}];

    let mut expected = VElement::new("div");
    expected.children = vec![
        VirtualNode::text(" "),
        VirtualNode::element("div"),
        VirtualNode::element("strong"),
        VirtualNode::text(" "),
    ];

    HtmlMacroTest {
        desc: "Vec of nodes",
        generated: html! { <div> { children } </div> },
        expected: expected.into(),
    }
    .test();
}

// FIXME: Move text tests into a separate file and import into book under
// html! section

#[test]
fn text_root_node() {
    HtmlMacroTest {
        desc: "Text as root node",
        generated: html! { some text },
        expected: VirtualNode::text("some text"),
    }
    .test()
}

/// Just make sure that this compiles since type is a keyword
#[test]
fn type_attribute() {
    html! { <link rel="stylesheet" type="text/css" href="/app.css" /> };
}

#[test]
fn text_variable_root() {
    let text = "hello world";

    HtmlMacroTest {
        desc: "Text variable root",
        generated: html! { { text } },
        expected: VirtualNode::text("hello world"),
    }
    .test()
}

#[test]
fn text_variable_child() {
    let text = "world";

    assert_eq!(
        &html! { <div>{ text }</div> }.to_string(),
        "<div>world</div>"
    )
}

#[test]
fn text_space_after_start_tag() {
    assert_eq!(
        &html! { <div> After Start Tag</div> }.to_string(),
        "<div> After Start Tag</div>"
    )
}

#[test]
fn text_space_before_end_tag() {
    assert_eq!(
        &html! { <div>Before End Tag </div> }.to_string(),
        "<div>Before End Tag </div>"
    )
}

#[test]
fn text_space_before_block() {
    let text = "Before Block";

    assert_eq!(
        &html! { <div> {text}</div> }.to_string(),
        "<div> Before Block</div>"
    )
}

#[test]
fn text_space_after_block() {
    let text = "Hello";

    assert_eq!(
        &html! { <div>{text} </div> }.to_string(),
        "<div>Hello </div>"
    )
}

#[test]
fn text_space_in_block_ignored() {
    let text = "Hello";

    assert_eq!(
        &html! { <div>{ text }</div> }.to_string(),
        "<div>Hello</div>"
    )
}

#[test]
fn text_multiple_text_no_space_between() {
    let hello = "Hello";
    let world = "World";

    assert_eq!(
        &html! { <div>{ hello }{ world }</div> }.to_string(),
        "<div>HelloWorld</div>"
    )
}

#[test]
fn text_multiple_text_space_between() {
    let hello = "Hello";
    let world = "World";

    assert_eq!(
        &html! { <div>{ hello } { world }</div> }.to_string(),
        "<div>Hello World</div>"
    )
}

#[test]
fn text_multiple_text_space_around() {
    let hello = "Hello";
    let world = "World";

    assert_eq!(
        &html! { <div> { hello }{ world } </div> }.to_string(),
        "<div> HelloWorld </div>"
    )
}

#[test]
fn text_multiple_text_space_between_around() {
    let hello = "Hello";
    let world = "World";

    assert_eq!(
        &html! { <div> { hello } { world } </div> }.to_string(),
        "<div> Hello World </div>"
    )
}

// Verify that all of our self closing tags work as both.
// Self closing tags can be written as either <tag> and <tag />
#[test]
fn self_closing_tag() {
    let mut expected = VElement::new("div");
    let children = vec![
        "area", "base", "br", "col", "hr", "img", "input", "link", "meta", "param", "command",
        "keygen", "source",
    ]
    .into_iter()
    .map(|tag| VirtualNode::element(tag))
    .collect();
    expected.children = children;

    let desc = &format!("Self closing tag without baskslash");
    HtmlMacroTest {
        desc,
        generated: html! {
            <div>
                <area> <base> <br> <col> <hr> <img> <input> <link> <meta> <param> <command>
                <keygen> <source>
            </div>
        },
        expected: expected.into(),
    }
    .test();

    let desc = &format!("Self closing tag with backslash");
    HtmlMacroTest {
        desc,
        generated: html! {
            <br />
        },
        expected: VirtualNode::element("br"),
    }
    .test();
}
