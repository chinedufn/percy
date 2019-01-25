#![feature(proc_macro_hygiene)]

use html_macro::html;
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
        generated: html! { <div></div> },
        expected: VirtualNode::new("div"),
        desc: "Empty div",
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
        generated: html! { <div id="hello-world"></div> },
        expected,
        desc: "One property",
    }
    .test();
}

//#[cfg(test)]
//mod tests {
//    use super::*;
//    use crate::VirtualNode;
//    use std::collections::HashMap;
//
//    struct HtmlMacroTest {
//        generated: VirtualNode,
//        expected: VirtualNode,
//        desc: &'static str,
//    }
//

//
//
//    #[test]
//    fn event() {
//        test(HtmlMacroTest {
//            generated: html! {
//                <div !onclick=|_: web_sys::MouseEvent| {},></div>
//            },
//            expected: html! {<div></div>},
//            desc: "Events are ignored in non wasm-32 targets",
//        });
//    }
//
//    #[test]
//    fn child_node() {
//        let mut expected = VirtualNode::new("div");
//        expected.children = Some(vec![VirtualNode::new("span")]);
//
//        test(HtmlMacroTest {
//            generated: html! { <div><span></span></div> },
//            expected,
//            desc: "Child node",
//        })
//    }
//
//    #[test]
//    fn sibling_child_nodes() {
//        let mut expected = VirtualNode::new("div");
//        expected.children = Some(vec![VirtualNode::new("span"), VirtualNode::new("b")]);
//
//        test(HtmlMacroTest {
//            generated: html! { <div><span></span><b></b></div> },
//            expected,
//            desc: "Sibling child nodes",
//        })
//    }
//
//    #[test]
//    fn three_nodes_deep() {
//        let mut child = VirtualNode::new("span");
//        child.children = Some(vec![VirtualNode::new("b")]);
//
//        let mut expected = VirtualNode::new("div");
//        expected.children = Some(vec![child]);
//
//        test(HtmlMacroTest {
//            generated: html! { <div><span><b></b></span></div> },
//            expected,
//            desc: "Nested 3 nodes deep",
//        })
//    }
//
//    #[test]
//    fn nested_text_node() {
//        let mut expected = VirtualNode::new("div");
//        expected.children = Some(vec![
//            VirtualNode::text("This is a text node"),
//            VirtualNode::text("More"),
//            VirtualNode::text("Text"),
//        ]);
//
//        test(HtmlMacroTest {
//            generated: html! { <div>{ "This is a text node" } {"More" "Text"}</div> },
//            expected,
//            desc: "Nested text nide",
//        });
//    }
//
//    #[test]
//    fn nested_macro() {
//        let child_2 = html! { <b></b> };
//
//        let mut expected = VirtualNode::new("div");
//        expected.children = Some(vec![VirtualNode::new("span"), VirtualNode::new("b")]);
//
//        test(HtmlMacroTest {
//            generated: html! { <div>{ html! { <span></span> } { child_2 } }</div> },
//            expected,
//            desc: "Nested macros",
//        });
//    }
//
//    #[test]
//    fn strings() {
//        let text1 = "This is a text node";
//        let text2 = text1.clone();
//
//        let mut expected = VirtualNode::new("div");
//        expected.children = Some(vec![
//            VirtualNode::text("This is a text node"),
//            VirtualNode::text("This is a text node"),
//        ]);
//
//        test(HtmlMacroTest {
//            generated: html! { <div>{ text1 text2 }</div> },
//            expected,
//            desc: "Creates text nodes",
//        });
//    }
//
//    #[test]
//    fn vec_of_nodes() {
//        let children = vec![html! { <div> </div>}, html! { <strong> </strong>}];
//
//        let mut expected = VirtualNode::new("div");
//        expected.children = Some(vec![VirtualNode::new("div"), VirtualNode::new("strong")]);
//
//        test(HtmlMacroTest {
//            generated: html! { <div> { children } </div> },
//            expected,
//            desc: "Vec of nodes",
//        });
//    }
//
//    #[test]
//    fn text_root_node() {
//        test(HtmlMacroTest {
//            generated: html! { { "some text" } },
//            expected: VirtualNode::text("some text"),
//            desc: "Text as root node",
//        })
//    }
//
//    fn test(macro_test: HtmlMacroTest) {
//        assert_eq!(
//            macro_test.generated, macro_test.expected,
//            "{}",
//            macro_test.desc
//        );
//
//        for (index, child) in macro_test
//            .expected
//            .children
//            .as_ref()
//            .unwrap()
//            .iter()
//            .enumerate()
//        {
//            assert_eq!(
//                child,
//                &macro_test.generated.children.as_ref().unwrap()[index],
//                "{}",
//                macro_test.desc
//            );
//        }
//    }
//}
