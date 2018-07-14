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
    Open,
    Close
}

/// A macro which returns a root `VirtualNode` given some HTML and Rust expressions.
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate virtual_dom_rs; fn main() {
///
/// let click_message = "I was clicked!";
/// let some_component = html! { <div !onclick=move || { println!("{}", click_message); },></div> };
///
/// let root_node = html! {
///  <div id="my-app", !onmouseenter=||{},>
///   <span> { "Hello world" } </span>
///   <b> { "How are" "you?" } </b>
///
///   { html! { <strong> { "nested macro call!" } </strong> } }
///   { some_component }
///  </div>
/// };
/// # }
/// ```
#[macro_export]
macro_rules! html {
    ($($remaining_html:tt)*) => {{
        let mut root_nodes: Vec<$crate::Rc<$crate::RefCell<$crate::VirtualNode>>> = vec![];

        {
            let mut active_node: Option<$crate::Rc<$crate::RefCell<$crate::VirtualNode>>> = None;
            let prev_tag_type: Option<$crate::TagType> = None;

            recurse_html! { active_node root_nodes prev_tag_type $($remaining_html)* };
        }

        $crate::Rc::try_unwrap(root_nodes.pop().unwrap()).unwrap().into_inner()
    }};
}

#[macro_export]
macro_rules! recurse_html {
    // The beginning of an element without any attributes.
    // For <div></div> this is
    // <div>
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident < $start_tag:ident > $($remaining_html:tt)*) => {
        let mut new_node = $crate::VirtualNode::new(stringify!($start_tag));
        let mut new_node = $crate::Rc::new($crate::RefCell::new(new_node));

        if $prev_tag_type == None {
            $root_nodes.push($crate::Rc::clone(&new_node));
        } else {
            $active_node.as_mut().unwrap().borrow_mut().children.push($crate::Rc::clone(&new_node));
            new_node.borrow_mut().parent = $active_node;
        }

        let mut $active_node = Some(new_node);

        let tag_type = Some($crate::TagType::Open);
        recurse_html! { $active_node $root_nodes tag_type $($remaining_html)* }
    };

    // The beginning of an element.
    // For <div id="10",> this is
    // <div
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident < $start_tag:ident $($remaining_html:tt)*) => {
        let mut new_node = $crate::VirtualNode::new(stringify!($start_tag));
        let mut new_node = $crate::Rc::new($crate::RefCell::new(new_node));

        if $prev_tag_type == None {
            $root_nodes.push($crate::Rc::clone(&new_node));
        } else {
            $active_node.as_mut().unwrap().borrow_mut().children.push($crate::Rc::clone(&new_node));
            new_node.borrow_mut().parent = $active_node;
        }

        $active_node = Some(new_node);

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
    //   $onclick=|| { do.something() }
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident ! $event_name:tt = $callback:expr, $($remaining_html:tt)*) => {
        $active_node.as_mut().unwrap().borrow_mut().events.0.insert(
            stringify!($event_name).to_string(),
            Box::new($callback)
        );

        recurse_html! { $active_node $root_nodes $prev_tag_type $($remaining_html)* }
    };

    // A block
    // for <div>{ Hello world }</div> this is:
    // "Hello world"
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident { $($child:expr)* } $($remaining_html:tt)*) => {
        $(
            let new_child = $crate::VirtualNode::from($child);
            let new_child = $crate::Rc::new($crate::RefCell::new(new_child));
            $active_node.as_mut().unwrap().borrow_mut().children.push($crate::Rc::clone(&new_child));
        )*

        recurse_html! { $active_node $root_nodes $prev_tag_type $($remaining_html)* }
    };

    // A closing tag for some associated opening tag name
    // For <div id="10",></div> this is:
    // </div>
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident < / $end_tag:ident > $($remaining_html:tt)*) => {
        let tag_type = Some($crate::TagType::Close);

        // Set the active node to the parent of the current active node that we just finished
        // processing
        let mut $active_node = $crate::Rc::clone(&$active_node.unwrap());
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
    use std::rc::Rc;
    use std::cell::RefCell;
    use VirtualNode;
    use std::collections::HashMap;

    #[test]
    fn empty_div() {
        let node = html!{
        <div></div>
        };

        let mut  expected_node = VirtualNode::new("div");

        assert_eq!(node, expected_node);
    }

    #[test]
    fn one_prop() {
        let node = html!{
        <div id="hello-world",></div>
        };

        let mut props = HashMap::new();
        props.insert("id".to_string(), "hello-world".to_string());
        let mut expected_node = VirtualNode::new("div");
        expected_node.props = props;

        assert_eq!(node, expected_node);
    }

    #[test]
    fn event() {
        struct TestStruct {
            closure_ran: bool
        };
        let test_struct = Rc::new(RefCell::new(TestStruct { closure_ran: false}));
        let test_struct_clone = Rc::clone(&test_struct);

        let node = html!{
        <div !onclick=move || {test_struct_clone.borrow_mut().closure_ran = true},></div>
        };

        assert_eq!(test_struct.borrow().closure_ran, false);

        node.events.0.get("onclick").unwrap()();

        assert_eq!(test_struct.borrow().closure_ran, true);
    }


    #[test]
    fn child_node() {
        let node = html!{
        <div><span></span></div>
        };

        let mut expected_node = VirtualNode::new("div");
        expected_node.children = vec![
            wrap(VirtualNode::new("span"))
        ];

        assert_eq!(node, expected_node);
        assert_eq!(expected_node.children.len(), 1);
    }

    #[test]
    fn sibling_child_nodes() {
        let node = html!{
        <div><span></span><b></b></div>
        };

        let mut expected_node = VirtualNode::new("div");
        expected_node.children = vec![
            wrap(VirtualNode::new("span")),
            wrap(VirtualNode::new("b"))
        ];

        assert_eq!(node, expected_node);
        assert_eq!(node.children.len(), 2);
    }

    #[test]
    fn three_nodes_deep () {
        let node = html!{
        <div><span><b></b></span></div>
        };

        let mut child = VirtualNode::new("span");
        child.children = vec![wrap(VirtualNode::new("b"))];

        let mut expected_node = VirtualNode::new("div");
        expected_node.children = vec![wrap(child)];

        assert_eq!(node, expected_node);
        assert_eq!(node.children.len(), 1, "1 Child");
    }

    #[test]
    fn nested_text_node() {
        let node = html!{
        <div>{ "This is a text node" } {"More" "Text"}</div>
        };

        let mut expected_node = VirtualNode::new("div");
        expected_node.children = vec![
            wrap(VirtualNode::text("This is a text node")),
            wrap(VirtualNode::text("More")),
            wrap(VirtualNode::text("Text")),
        ];

        assert_eq!(node, expected_node);
        assert_eq!(node.children.len(), 3, "3 text node children");

        // TODO: assert_same_children(node, expected_node)
        for (index, child) in node.children.iter().enumerate() {
            assert_eq!(child, &expected_node.children[index]);
        }
    }

    #[test]
    fn nested_macro() {
        let child_2 = html! { <b></b> };

        let node = html!{
        <div>{ html! { <span></span> } { child_2 } }</div>
        };

        let mut expected_node = VirtualNode::new("div");
        expected_node.children = vec![
            wrap(VirtualNode::new("span")),
            wrap(VirtualNode::new("b")),
        ];

        assert_eq!(node, expected_node);
    }

    fn wrap (v: VirtualNode) -> Rc<RefCell<VirtualNode>> {
        Rc::new(RefCell::new(v))
    }
}