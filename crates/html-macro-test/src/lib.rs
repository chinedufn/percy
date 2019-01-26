#![feature(proc_macro_hygiene)]

use html_macro::{html, text};
use std::collections::HashMap;
use virtual_node::VirtualNode;

struct HtmlMacroTest {
    desc: &'static str,
    generated: VirtualNode,
    expected: VirtualNode,
}

impl HtmlMacroTest {
    fn test(self) {
        assert_eq!(self.generated, self.expected, "{}", self.desc);

        for (index, child) in self.expected.children.as_ref().unwrap().iter().enumerate() {
            assert_eq!(
                child,
                &self.generated.children.as_ref().unwrap()[index],
                "{}",
                self.desc
            );
        }
    }
}

#[test]
fn empty_div() {
    HtmlMacroTest {
        desc: "Empty div",
        generated: html! { <div></div> },
        expected: VirtualNode::new("div"),
    }
    .test();
}

#[test]
fn one_prop() {
    let mut props = HashMap::new();
    props.insert("id".to_string(), "hello-world".to_string());
    let mut expected = VirtualNode::new("div");
    expected.props = props;

    HtmlMacroTest {
        desc: "One property",
        generated: html! { <div id="hello-world"></div> },
        expected,
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
    let mut expected = VirtualNode::new("div");
    expected.children = Some(vec![VirtualNode::new("span")]);

    HtmlMacroTest {
        desc: "Child node",
        generated: html! { <div><span></span></div> },
        expected,
    }
    .test();
}

#[test]
fn sibling_child_nodes() {
    let mut expected = VirtualNode::new("div");
    expected.children = Some(vec![VirtualNode::new("span"), VirtualNode::new("b")]);

    HtmlMacroTest {
        desc: "Sibling child nodes",
        generated: html! { <div><span></span><b></b></div> },
        expected,
    }
    .test();
}

#[test]
fn three_nodes_deep() {
    let mut child = VirtualNode::new("span");
    child.children = Some(vec![VirtualNode::new("b")]);

    let mut expected = VirtualNode::new("div");
    expected.children = Some(vec![child]);

    HtmlMacroTest {
        desc: "Nested 3 nodes deep",
        generated: html! { <div><span><b></b></span></div> },
        expected,
    }
    .test()
}

#[test]
fn sibling_text_nodes() {
    let mut expected = VirtualNode::new("div");
    expected.children = Some(vec![VirtualNode::text("This is a text node")]);

    HtmlMacroTest {
        desc: "Nested text node",
        generated: html! { <div>This is a text node</div> },
        expected,
    }
    .test();
}

#[test]
fn nested_macro() {
    let child_2 = html! { <b></b> };

    let mut expected = VirtualNode::new("div");
    expected.children = Some(vec![VirtualNode::new("span"), VirtualNode::new("b")]);

    HtmlMacroTest {
        desc: "Nested macros",
        generated: html! {
          <div>
            { html! { <span></span> } }
            { child_2 }
          </div>
        },
        expected,
    }
    .test();
}

#[test]
fn block_root() {
    let em = html! { <em></em> };

    let mut expected = VirtualNode::new("em");

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

    let mut expected = VirtualNode::new("div");
    expected.children = Some(vec![
        VirtualNode::text("A bit of text"),
        VirtualNode::new("ul"),
    ]);

    HtmlMacroTest {
        desc: "Text node next to a block",
        generated: html! {
          <div>
            A bit of text
            { child }
          </div>
        },
        expected,
    }
    .test();
}

#[test]
fn punctuation_comma() {
    let text = "Hello, World";

    HtmlMacroTest {
        desc: "Comma",
        generated: html! { Hello, World},
        expected: VirtualNode::text(&text),
    }
        .test()
}

#[test]
fn punctuation_exclamation() {
    let text = "Hello World!";

    HtmlMacroTest {
        desc: "Exclamation point",
        generated: html! { Hello World! },
        expected: VirtualNode::text(&text),
    }
        .test()
}

#[test]
fn punctuation_period() {
    let text = "Hello.";

    HtmlMacroTest {
        desc: "Period",
        generated: html! { Hello. },
        expected: VirtualNode::text(&text),
    }
        .test()
}

#[test]
fn vec_of_nodes() {
    let children = vec![html! { <div> </div>}, html! { <strong> </strong>}];

    let mut expected = VirtualNode::new("div");
    expected.children = Some(vec![VirtualNode::new("div"), VirtualNode::new("strong")]);

    HtmlMacroTest {
        desc: "Vec of nodes",
        generated: html! { <div> { children } </div> },
        expected,
    }
    .test();
}

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
    html! { <link rel="stylesheet" type="text/css" href="/app.css"></link> };
}

#[test]
fn text_macro() {
    let text_var = "some text";

    HtmlMacroTest {
        desc: "text! creates text from variables",
        generated: text!(text_var),
        expected: VirtualNode::text("some text"),
    }
    .test()
}

// TODO: Support self closing tags such as <b />
