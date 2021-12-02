//! This is a catch-all module to place new tests as we go.
//!
//! Over time we'll pull tests out of here and organize them.
//!
//! For example - there is a `text_tests.rs` module where all of our text node related
//! tests live.

use html_macro::html;
use std::collections::HashMap;
use virtual_node::{IterableNodes, VElement, VText, View, VirtualNode};

pub(crate) struct HtmlMacroTest {
    pub generated: VirtualNode,
    pub expected: VirtualNode,
}

impl HtmlMacroTest {
    /// Ensure that the generated and the expected virtual node are equal.
    pub fn test(self) {
        assert_eq!(self.expected, self.generated);
    }
}

#[test]
fn empty_div() {
    HtmlMacroTest {
        generated: html! { <div></div> },
        expected: VirtualNode::element("div"),
    }
    .test();
}

#[test]
fn one_attr() {
    let mut attrs = HashMap::new();
    let id = "hello-world".into();
    attrs.insert("id".to_string(), id);
    let mut expected = VElement::new("div");
    expected.attrs = attrs;

    HtmlMacroTest {
        generated: html! { <div id="hello-world"></div> },
        expected: expected.into(),
    }
    .test();
}

#[test]
fn child_node() {
    let mut expected = VElement::new("div");
    expected.children = vec![VirtualNode::element("span")];

    HtmlMacroTest {
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
        generated: html! { <div><span></span><b></b></div> },
        expected: expected.into(),
    }
    .test();
}

/// Nested 3 nodes deep
#[test]
fn three_nodes_deep() {
    let mut child = VElement::new("span");
    child.children = vec![VirtualNode::element("b")];

    let mut expected = VElement::new("div");
    expected.children = vec![child.into()];

    HtmlMacroTest {
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
        generated: html! { <div>This is a text node</div> },
        expected: expected.into(),
    }
    .test();
}

#[test]
fn nested_macro() {
    let child_2 = html! { <b></b> };

    let mut expected = VElement::new("div");
    expected.children = vec![VirtualNode::element("span"), VirtualNode::element("b")];

    HtmlMacroTest {
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

/// If the first thing we see is a block then we grab whatever is inside it.
#[test]
fn block_root() {
    let em = html! { <em></em> };

    let expected = VirtualNode::element("em");

    HtmlMacroTest {
        generated: html! {
            { em }
        },
        expected,
    }
    .test();
}

/// Text followed by a block
#[test]
fn text_next_to_block() {
    let child = html! { <ul></ul> };

    let mut expected = VElement::new("div");
    expected.children = vec![
        VirtualNode::text(" A bit of text "),
        VirtualNode::element("ul"),
    ];

    HtmlMacroTest {
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

/// Ensure that we maintain the correct spacing around punctuation tokens, since
/// they resolve into a separate TokenStream during parsing.
#[test]
fn punctuation_token() {
    let text = "Hello, World";

    HtmlMacroTest {
        generated: html! { Hello, World },
        expected: VirtualNode::text(text),
    }
    .test()
}

#[test]
fn vec_of_nodes() {
    let children = vec![html! { <div> </div>}, html! { <strong> </strong>}];

    let mut expected = VElement::new("div");
    expected.children = vec![VirtualNode::element("div"), VirtualNode::element("strong")];

    HtmlMacroTest {
        generated: html! { <div> { children } </div> },
        expected: expected.into(),
    }
    .test();
}

/// Just make sure that this compiles since as, async, for, loop, and type are keywords
#[test]
fn keyword_attribute() {
    html! { <link rel="prefetch" href="/style.css" as="style" /> };
    html! { <script src="/app.js" async="async" /> };
    html! { <label for="username">Username:</label> };
    html! { <audio loop="loop"><source src="/beep.mp3" type="audio/mpeg" /></audio> };
    html! { <link rel="stylesheet" type="text/css" href="/app.css" /> };
}

/// For unquoted text apostrophes should be parsed correctly
#[test]
fn apostrophe() {
    assert_eq!(html! { Aren't }, VText::new("Aren't").into());
    assert_eq!(html! { Aren'ttt }, VText::new("Aren'ttt").into());
}

/// Verify that all of our self closing tags work without backslashes.
#[test]
fn self_closing_tag_without_backslash() {
    let mut expected = VElement::new("div");
    let children = vec![
        "area", "base", "br", "col", "hr", "img", "input", "link", "meta", "param", "command",
        "keygen", "source",
    ]
    .into_iter()
    .map(|tag| VirtualNode::element(tag))
    .collect();
    expected.children = children;

    HtmlMacroTest {
        generated: html! {
            <div>
                <area> <base> <br> <col> <hr> <img> <input> <link> <meta> <param> <command>
                <keygen> <source>
            </div>
        },
        expected: expected.into(),
    }
    .test();
}

/// Verify that our self closing tags work with backslashes
#[test]
fn self_closing_tag_with_backslace() {
    HtmlMacroTest {
        generated: html! {
            <br />
        },
        expected: VirtualNode::element("br"),
    }
    .test();
}

#[test]
fn if_true_block() {
    let child_valid = html! { <b></b> };
    let child_invalid = html! { <i></i> };

    let mut expected = VElement::new("div");
    expected.children = vec![VirtualNode::element("b")];

    HtmlMacroTest {
        generated: html! {
          <div>
            {if true {child_valid} else {child_invalid}}
          </div>
        },
        expected: expected.into(),
    }
    .test();
}

#[test]
fn if_false_block() {
    let child_valid = html! { <b></b> };
    let child_invalid = html! { <i></i> };

    let mut expected = VElement::new("div");
    expected.children = vec![VirtualNode::element("i")];

    HtmlMacroTest {
        generated: html! {
          <div>
            {if false {
                child_valid
            } else {
                child_invalid
            }}
          </div>
        },
        expected: expected.into(),
    }
    .test();
}

#[test]
fn single_branch_if_true_block() {
    let child_valid = html! { <b></b> };

    let mut expected = VElement::new("div");
    expected.children = vec![VirtualNode::element("b")];

    HtmlMacroTest {
        generated: html! {
          <div>{if true {child_valid}}</div>
        },
        expected: expected.into(),
    }
    .test();
}

#[test]
fn single_branch_if_false_block() {
    let child_valid = html! { <b></b> };

    let mut expected = VElement::new("div");
    expected.children = vec![VirtualNode::text("")];

    HtmlMacroTest {
        generated: html! {
          <div>{if false {child_valid}}</div>
        },
        expected: expected.into(),
    }
    .test();
}

#[test]
fn custom_component_props() {
    struct Counter {
        count: u8,
    }

    impl View for Counter {
        fn render(&self) -> VirtualNode {
            html! {
                <span>Counter = {format!("{}", self.count)}</span>
            }
        }
    }

    let mut expected = VElement::new("div");
    let mut child = VElement::new("span");
    child.children = vec![VirtualNode::text("Counter = "), VirtualNode::text("1")];
    expected.children = vec![child.into()];

    HtmlMacroTest {
        generated: html! {
          <div><Counter count={1}/></div>
        },
        expected: expected.into(),
    }
    .test();
}

#[test]
fn custom_component_children() {
    struct Child;

    impl View for Child {
        fn render(&self) -> VirtualNode {
            html! {
                <span></span>
            }
        }
    }

    let mut expected = VElement::new("div");
    let mut child = VElement::new("span");
    child.children = vec![VirtualNode::text("Hello World")];
    expected.children = vec![child.into()];

    HtmlMacroTest {
        generated: html! {
          <div>
            <Child>Hello World</Child>
          </div>
        },
        expected: expected.into(),
    }
    .test();
}

/// Verify that we can properly render an empty list of virtual nodes that has a space after it.
/// Before this test our code generation was assuming that all lists had more than one element when
/// checking if we needed to insert space after an element.
#[test]
fn space_before_and_after_empty_list() {
    let elements: Vec<VirtualNode> = Vec::new();

    HtmlMacroTest {
        generated: html! {<div> {elements} </div>},
        expected: html! {<div> </div>},
    }
    .test()
}

/// Verify that an Option::None virtual node gets ignored.
#[test]
fn option_none() {
    let element: Option<VirtualNode> = None;

    HtmlMacroTest {
        generated: html! {<div> {element} </div>},
        expected: html! {<div> </div>},
    }
    .test()
}

/// Verify that an Some(VirtualNode) gets rendered.
#[test]
fn option_some() {
    let element: Option<VirtualNode> = Some(VirtualNode::element("em"));

    HtmlMacroTest {
        generated: html! {<div> {element} </div>},
        expected: html! {<div> <em></em> </div>},
    }
    .test()
}

/// Verify that our macro to generate IterableNodes implementations for numbers works.
#[test]
fn numbers() {
    let num = 3u8;
    let ref_num = &4u8;

    HtmlMacroTest {
        generated: html! {<div> {num} {ref_num} </div>},
        expected: html! {<div> {"3"} {"4"} </div>},
    }
    .test()
}
