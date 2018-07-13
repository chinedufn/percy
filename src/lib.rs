//!

use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq, Default))]
pub struct VirtualNode {
    tag: String,
    props: HashMap<String, String>,
    events: Events,
    children: Vec<VirtualNode>,
    /// Some(String) if this is a [text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
    /// When patching these into a real DOM these use `document.createTextNode(text)`
    text: Option<String>,
}

#[cfg_attr(test, derive(Default))]
pub struct Events(HashMap<String, Box<Fn() -> ()>>);

impl PartialEq for Events {
    // Once you set events on an element you can't change them, so we don't factor them
    // into our PartialEq
    fn eq(&self, rhs: &Self) -> bool {
       true
    }
}

impl fmt::Debug for Events {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let events: String = self.0.keys().map(|key| format!("{} ", key)).collect();
        write!(f, "{}", events)
    }
}

impl VirtualNode {
    fn new (tag: &str) -> VirtualNode {
        let props = HashMap::new();
        let events = Events(HashMap::new());
        VirtualNode {
            tag: tag.to_string(),
            props,
            events,
            children: vec![],
            text: None
        }
    }
}

pub fn createElement(node: &VirtualNode) {
    // document.createElement(node.type)
}

#[cfg_attr(test, derive(Debug))]
struct NodeParser {
    // TODO: current_node -> node
    current_node: Option<VirtualNode>,
    parent: Option<Box<NodeParser>>,
}

// TODO: Move to html_macro.rs along w/ tests
#[macro_export]
macro_rules! html {
    ($($remaining_html:tt)*) => {{
        // TODO: Rename to parsed_node
        let mut pnt = NodeParser {
        // Probably also need a reference to the sibling so that we can traverse
        // backwards through siblings in a nested while loop..
        // push to the list then reverse it..
            current_node: None,
            parent: None,
        };

        recurse_html! { pnt $($remaining_html)* };

        while (pnt.parent.is_some() && pnt.parent.as_ref().unwrap().current_node.is_some()) {
            let node = pnt.current_node.take().unwrap();
            pnt = *pnt.parent.unwrap();
            pnt.current_node.as_mut().unwrap().children.push(node);
        };

        pnt.current_node.unwrap()
    }};
}

#[macro_export]
macro_rules! recurse_html {
    // The beginning of an element without any attributes.
    // For <div></div> this is
    // <div>
    ($pnt:ident < $start_tag:ident > $($remaining_html:tt)*) => {
        let current_node = VirtualNode::new(stringify!($start_tag));

        $pnt = NodeParser {
            current_node: Some(current_node),
            parent: Some(Box::new($pnt)),
        };

        recurse_html! { $pnt $($remaining_html)* }
    };

    // The beginning of an element.
    // For <div id="10",> this is
    // <div
    ($pnt:ident < $start_tag:ident $($remaining_html:tt)*) => {
        let current_node = VirtualNode::new(stringify!($start_tag));

        $pnt = NodeParser {
            current_node: Some(current_node),
            parent: Some(Box::new($pnt)),
        };

        recurse_html! { $pnt $($remaining_html)* }
    };

    // The end of an opening tag.
    // For <div id="10",> this is:
    //  >
    ($pnt:ident > $($remaining_html:tt)*) => {
        recurse_html! { $pnt $($remaining_html)* }
    };

    // A property
    // For <div id="10",> this is:
    // id = "10",
    ($pnt:ident $prop_name:tt = $prop_value:expr, $($remaining_html:tt)*) => {
        $pnt.current_node.as_mut().unwrap().props.insert(
            stringify!($prop_name).to_string(),
            $prop_value.to_string()
        );

        recurse_html! { $pnt $($remaining_html)* }
    };

    // An event
    // for <div $onclick=|| { do.something(); },></div> ths is:
    //   $onclick=|| { do.something() }
    ($pnt:ident ! $event_name:tt = $callback:expr, $($remaining_html:tt)*) => {
        $pnt.current_node.as_mut().unwrap().events.0.insert(
            stringify!($event_name).to_string(),
            Box::new($callback)
        );

        recurse_html! { $pnt $($remaining_html)* }
    };


    // A closing tag for some associated opening tag name
    // For <div id="10",></div> this is:
    // </div>
    ($pnt:ident < / $end_tag:ident > $($remaining_html:tt)*) => {
        recurse_html! { $pnt $($remaining_html)* }
    };

    // No more HTML remaining. We're done!
    ($pnt:ident) => {
    };

    // TODO: README explains that props must end with commas
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::rc::Rc;
    use std::cell::RefCell;

    #[test]
    fn empty_div() {
        let node = html!{
        <div></div>
        };

        let expected_node = VirtualNode {
            tag: "div".to_string(),
            ..VirtualNode::default()
        };

        assert_eq!(node, expected_node);
    }

    #[test]
    fn one_prop() {
        let node = html!{
        <div id="hello-world",></div>
        };

        let mut props = HashMap::new();
        props.insert("id".to_string(), "hello-world".to_string());
        let expected_node = VirtualNode {
            tag: "div".to_string(),
            props,
            ..VirtualNode::default()
        };

        assert_eq!(node, expected_node);
    }

    #[test]
    fn event() {
        struct TestStruct {
            closure_ran: bool
        };
        // TODO: Rc<>
        let mut test_struct = Rc::new(RefCell::new(TestStruct { closure_ran: false}));
        let mut test_struct_clone = Rc::clone(&test_struct);

        let node = html!{
        <div !onclick=move || {test_struct_clone.borrow_mut().closure_ran = true},></div>
        };


        assert_eq!(test_struct.borrow().closure_ran, false);

        node.events.0.get("onclick").unwrap()();

        assert_eq!(test_struct.borrow().closure_ran, true);
    }


    #[test]
    fn child_node() {
        let mut node = html!{
        <div><span></span></div>
        };

        let child = VirtualNode {
            tag: "span".to_string(),
            ..VirtualNode::default()
        };
        let children = vec![child];

        let expected_node = VirtualNode {
            tag: "div".to_string(),
            children,
            ..VirtualNode::default()
        };

        assert_eq!(node, expected_node);
    }

    #[test]
    fn sibling_child_nodes() {
        let mut node = html!{
        <div><span></span><b></b></div>
        };

        let sibling1 = VirtualNode {
            tag: "span".to_string(),
            ..VirtualNode::default()
        };
        let sibling2 = VirtualNode {
            tag: "b".to_string(),
            ..VirtualNode::default()
        };
        let children = vec![sibling1, sibling2];

        let expected_node = VirtualNode {
            tag: "div".to_string(),
            children,
            ..VirtualNode::default()
        };

        assert_eq!(node, expected_node);
    }
}
