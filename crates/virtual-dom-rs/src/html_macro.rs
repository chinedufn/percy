//! The html_macro module exposes an `html!` macro that is used to generate `VirtualNode`'s
//! that will eventually get rendered into DOM nodes on the client side, or String's
//! if you're on the server side.

use js_sys::Function;
use wasm_bindgen::prelude::Closure;

/// When parsing our HTML we keep track of whether the last tag that we saw was an open or
/// close tag.
///
/// We use this information whenever we encounter a new open tag.
///
/// If the previous tag was an Open tag, then this new open tag is the child of the previous tag.
///
/// For example, in `<foo><bar></bar></foo>` `<bar>` is the child of `<foo>` since the last tag
/// was an open tag `<foo>`
///
/// If the previous tag was a Close tag, then this new open tag is the sibling of the previous
/// tag, so they share the same parent.
///
/// For example, in `<foo><bar></bar><bing></bing>` <bing> is a the child of "</bar>"'s parent since
/// </bar> is a closing tag. Soo `<bing>`'s parent is `<foo>`
#[derive(PartialEq)]
#[cfg_attr(test, derive(Debug))]
pub enum TagType {
    /// An opening HTML tag
    ///  ex: "<div" or "<div>"
    ///
    Open,
    /// A closing HTML tag
    ///  ex: "</div>"
    Close,
}

/// A macro which returns a root VirtualNode given some HTML and Rust expressions.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate virtual_dom_rs;  fn main() {
///
/// use virtual_dom_rs::VirtualNode;
///
/// let click_message = "I was clicked!";
/// let some_component = html! { <div !onclick=move |_ev| { println!("{}", click_message); },></div> };
///
/// // Create lists of nodes from data!
/// let list: Vec<VirtualNode> = [0, 1, 2].iter().map(|index| {
///   let index = index.to_string();
///   html! {
///     <div key="unique-key-{index}",>
///       <h1> {"This is node number"} {index} </h1>
///       <strong> {"Keys in lists help performance"} </strong>
///       <em> { "But they're optional" } </em>
///     </div>
///   }
/// }).collect();
///
/// let root_node = html! {
///  <div id="my-app", !onmouseenter=|_ev|{},>
///   <span> { "Hello world" } </span>
///   <b> { "How are" "you?" } </b>
///
///   { html! { <strong> { "nested macro call!" } </strong> } }
///
///   { some_component }
///
///   { list }
///
///   // You can have
///   /*  comments in your html! */
///  </div>
/// };
/// # }
/// ```
///
/// # TODO
///
/// Create a separate macro that works with anything that implements VNode
///
/// ```ignore
/// struct MyCustomVirtualNode;
/// impl VNode for MyCustomVirtualNode {
///   ...
/// }
///
/// html_generic ! { MyCustomVirtualNode <div> <span></span> </div> };
/// ```
///
/// Then make `html! { <div></div> }` call `html_generic! { $crate::VirtualNode <div></div> }`.
///
/// This would allow anyone to use the html_generic! macro to power their own virtual dom
/// implementation!
#[macro_export]
macro_rules! html {
    ($($remaining_html:tt)*) => {{
        let mut root_nodes: Vec<$crate::Rc<$crate::RefCell<$crate::ParsedVirtualNode>>> = vec![];

        {
            let mut active_node: Option<$crate::Rc<$crate::RefCell<$crate::ParsedVirtualNode>>> = None;

            let prev_tag_type: Option<$crate::TagType> = None;

            recurse_html! { active_node root_nodes prev_tag_type $($remaining_html)* };
            drop(&active_node);
            drop(&prev_tag_type);
        }

        $crate::VirtualNode::from($crate::Rc::try_unwrap(root_nodes.pop().unwrap()).unwrap().into_inner())
    }};
}

/// Powers the html! macro
#[macro_export]
macro_rules! recurse_html {
    // The beginning of an element without any attributes.
    // For <div></div> this is
    // <div>
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident < $start_tag:ident > $($remaining_html:tt)*) => {
        #[allow(unused_mut)]
        let mut new_node = $crate::ParsedVirtualNode::new(stringify!($start_tag));
        #[allow(unused_mut)]
        let mut new_node = $crate::Rc::new($crate::RefCell::new(new_node));

        if $prev_tag_type == None {
            $root_nodes.push($crate::Rc::clone(&new_node));
        } else {
            $active_node.as_mut().unwrap().borrow_mut().children.as_mut().unwrap().push($crate::Rc::clone(&new_node));
            new_node.borrow_mut().parent = $active_node;
        }

        #[allow(unused_mut)]
        let mut $active_node = Some(new_node);

        #[allow(unused)]
        let tag_type = Some($crate::TagType::Open);
        recurse_html! { $active_node $root_nodes tag_type $($remaining_html)* }
    };

    // The beginning of an element.
    // For <div id="10",> this is
    // <div
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident < $start_tag:ident $($remaining_html:tt)*) => {
        #[allow(unused_mut)]
        let mut new_node = $crate::ParsedVirtualNode::new(stringify!($start_tag));
        #[allow(unused_mut)]
        let mut new_node = $crate::Rc::new($crate::RefCell::new(new_node));

        if $prev_tag_type == None {
            $root_nodes.push($crate::Rc::clone(&new_node));
        } else {
            $active_node.as_mut().unwrap().borrow_mut().children.as_mut().unwrap().push($crate::Rc::clone(&new_node));
            new_node.borrow_mut().parent = $active_node;
        }

        $active_node = Some(new_node);

        #[allow(unused)]
        let tag_type = Some($crate::TagType::Open);
        recurse_html! { $active_node $root_nodes tag_type $($remaining_html)* }
    };

    // The end of an opening tag.
    // For <div id="10",> this is:
    //  >
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident > $($remaining_html:tt)*) => {
        recurse_html! { $active_node $root_nodes $prev_tag_type $($remaining_html)* }
    };

    // A property
    // For <div id="10",> this is:
    // id = "10",
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident $prop_name:ident = $prop_value:expr, $($remaining_html:tt)*) => {
        $active_node.as_mut().unwrap().borrow_mut().props.insert(
            stringify!($prop_name).to_string(),
            $prop_value.to_string()
        );

        recurse_html! { $active_node $root_nodes $prev_tag_type $($remaining_html)* }
    };


    // An event
    // for <div $onclick=|| { do.something(); },></div> ths is:
    //   $onclick=|| { do.something(); }
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident ! $event_name:tt = $callback:expr, $($remaining_html:tt)*) => {
        // Closure::wrap is not implemented on non wasm32 targets
        #[cfg(target_arch = "wasm32")]
        {
            let closure = $crate::Closure::wrap(Box::new($callback) as Box<FnMut(_)>);
            let closure = $crate::Rc::new(closure);

            $active_node.as_mut().unwrap().borrow_mut().custom_events.0.insert(
                stringify!($event_name).to_string(),
                closure
            );
        }

        recurse_html! { $active_node $root_nodes $prev_tag_type $($remaining_html)* }
    };

    // A block
    // for <div>{ "Hello world" }</div> this is:
    // "Hello world"
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident { $($child:expr)* } $($remaining_html:tt)*) => {
        $(
            // A user can pass in different types. A string.. a node.. a vector of nodes.. So we
            // convert whatever they passed in into a ParsedVirtualNode.
            let new_child = $crate::ParsedVirtualNode::from($child);

            // If the user passed in a vector of nodes we converted it into a ParsedVirtualNode
            // that has a special tag that let's us know that what we really want is to iterate over
            // the vector of nodes that they passed in.
            // So.. in short.. we stored the vector of nodes that they passed in as children of
            // some temporary ParsedVirtualNode, and now we're pulling them out and appending them
            // to their rightful parent.
            if new_child.tag == "__VEC_OF_CHILDREN__".to_string() {
                for node_from_vec in new_child.children.unwrap() {
                    $active_node.as_mut().unwrap().borrow_mut().children.as_mut().unwrap().push(
                      $crate::Rc::clone(&node_from_vec)
                    );
                }
            } else {
               let new_child = $crate::Rc::new($crate::RefCell::new(new_child));

               if $root_nodes.len() == 1 {
                   $active_node.as_mut().unwrap().borrow_mut().children.as_mut().unwrap().push($crate::Rc::clone(&new_child));
               } else {
                   // This handles the case when the root node is inside of a block
                   $root_nodes.push($crate::Rc::clone(&new_child));
                   $active_node = Some(new_child);
               }
            }
        )*

        recurse_html! { $active_node $root_nodes $prev_tag_type $($remaining_html)* }
    };

    // A closing tag for some associated opening tag name
    // For <div id="10",></div> this is:
    // </div>
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident < / $end_tag:ident > $($remaining_html:tt)*) => {
        #[allow(unused)]
        let tag_type = Some($crate::TagType::Close);

        // Set the active node to the parent of the current active node that we just finished
        // processing
        #[allow(unused_mut)]
        let mut $active_node = $crate::Rc::clone(&$active_node.unwrap());
        #[allow(unused_mut)]
        let mut $active_node = $active_node.borrow_mut().parent.take();

        recurse_html! { $active_node $root_nodes tag_type $($remaining_html)* }
    };

    // No more HTML remaining. We're done!
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident) => {
    };

    // TODO: README explains that props must end with commas
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::VirtualNode;
    use std::collections::HashMap;

    struct HTMLMacroTest {
        generated: VirtualNode,
        expected: VirtualNode,
        desc: &'static str,
    }

    #[test]
    fn empty_div() {
        test(HTMLMacroTest {
            generated: html! { <div></div> },
            expected: VirtualNode::new("div"),
            desc: "Empty div",
        })
    }

    #[test]
    fn one_prop() {
        let mut props = HashMap::new();
        props.insert("id".to_string(), "hello-world".to_string());
        let mut expected = VirtualNode::new("div");
        expected.props = props;

        test(HTMLMacroTest {
            generated: html! { <div id="hello-world",></div> },
            expected,
            desc: "One property",
        });
    }

    #[test]
    fn event() {
        test(HTMLMacroTest {
            generated: html! {
                <div !onclick=|_: web_sys::MouseEvent| {},></div>
            },
            expected: html! {<div></div>},
            desc: "Events are ignored in non wasm-32 targets",
        });
    }

    #[test]
    fn child_node() {
        let mut expected = VirtualNode::new("div");
        expected.children = Some(vec![VirtualNode::new("span")]);

        test(HTMLMacroTest {
            generated: html! { <div><span></span></div> },
            expected,
            desc: "Child node",
        })
    }

    #[test]
    fn sibling_child_nodes() {
        let mut expected = VirtualNode::new("div");
        expected.children = Some(vec![VirtualNode::new("span"), VirtualNode::new("b")]);

        test(HTMLMacroTest {
            generated: html! { <div><span></span><b></b></div> },
            expected,
            desc: "Sibling child nodes",
        })
    }

    #[test]
    fn three_nodes_deep() {
        let mut child = VirtualNode::new("span");
        child.children = Some(vec![VirtualNode::new("b")]);

        let mut expected = VirtualNode::new("div");
        expected.children = Some(vec![child]);

        test(HTMLMacroTest {
            generated: html! { <div><span><b></b></span></div> },
            expected,
            desc: "Nested 3 nodes deep",
        })
    }

    #[test]
    fn nested_text_node() {
        let mut expected = VirtualNode::new("div");
        expected.children = Some(vec![
            VirtualNode::text("This is a text node"),
            VirtualNode::text("More"),
            VirtualNode::text("Text"),
        ]);

        test(HTMLMacroTest {
            generated: html! { <div>{ "This is a text node" } {"More" "Text"}</div> },
            expected,
            desc: "Nested text nide",
        });
    }

    #[test]
    fn nested_macro() {
        let child_2 = html! { <b></b> };

        let mut expected = VirtualNode::new("div");
        expected.children = Some(vec![VirtualNode::new("span"), VirtualNode::new("b")]);

        test(HTMLMacroTest {
            generated: html! { <div>{ html! { <span></span> } { child_2 } }</div> },
            expected,
            desc: "Nested macros",
        });
    }

    #[test]
    fn strings() {
        let text1 = "This is a text node";
        let text2 = text1.clone();

        let mut expected = VirtualNode::new("div");
        expected.children = Some(vec![
            VirtualNode::text("This is a text node"),
            VirtualNode::text("This is a text node"),
        ]);

        test(HTMLMacroTest {
            generated: html! { <div>{ text1 text2 }</div> },
            expected,
            desc: "Creates text nodes",
        });
    }

    #[test]
    fn vec_of_nodes() {
        let children = vec![html! { <div> </div>}, html! { <strong> </strong>}];

        let mut expected = VirtualNode::new("div");
        expected.children = Some(vec![VirtualNode::new("div"), VirtualNode::new("strong")]);

        test(HTMLMacroTest {
            generated: html! { <div> { children } </div> },
            expected,
            desc: "Vec of nodes",
        });
    }

    #[test]
    fn text_root_node() {
        test(HTMLMacroTest {
            generated: html! { { "some text" } },
            expected: VirtualNode::text("some text"),
            desc: "Text as root node",
        })
    }

    // TODO: Support Option<VirtualNode> as a child. When see wee that we do nothing. This
    // would allow views to return `None` if nothing should be rendered.

    fn test(macro_test: HTMLMacroTest) {
        assert_eq!(
            macro_test.generated, macro_test.expected,
            "{}",
            macro_test.desc
        );

        for (index, child) in macro_test
            .expected
            .children
            .as_ref()
            .unwrap()
            .iter()
            .enumerate()
        {
            assert_eq!(
                child,
                &macro_test.generated.children.as_ref().unwrap()[index],
                "{}",
                macro_test.desc
            );
        }
    }
}
