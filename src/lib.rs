//!

use std::collections::HashMap;
use std::fmt;
use std::cell::RefCell;
use std::rc::Rc;

pub struct VirtualNode {
    tag: String,
    props: HashMap<String, String>,
    events: Events,
    children: Vec<Rc<RefCell<VirtualNode>>>,
    parent: Option<Rc<RefCell<VirtualNode>>>,
    /// Some(String) if this is a [text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
    /// When patching these into a real DOM these use `document.createTextNode(text)`
    text: Option<String>,
}

impl fmt::Debug for VirtualNode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "VirtualNode | tag: {}, props: {:#?}, text: {:#?}, children: {:#?} |", self.tag, self.props, self.text, self.children)
    }
}

impl PartialEq for VirtualNode {
    fn eq(&self, rhs: &Self) -> bool {
        self.tag == rhs.tag &&
            self.props == rhs.props &&
            self.text == rhs.text
    }
}

//impl Drop for VirtualNode {
//    fn drop(&mut self) {
//        self.parent = None;
//    }
//}

// TODO: No longer need this since we implement partialeq ourselves for VirtualNode
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
            parent: None,
            text: None
        }
    }
}

#[derive(PartialEq)]
#[cfg_attr(test, derive(Debug))]
enum TagType {
    Open,
    Close
}

// TODO: Move to html_macro.rs along w/ tests
#[macro_export]
macro_rules! html {
    ($($remaining_html:tt)*) => {{
        let mut root_nodes: Vec<Rc<RefCell<VirtualNode>>> = vec![];
        let mut active_node: Option<Rc<RefCell<VirtualNode>>> = None;
        let prev_tag_type: Option<TagType> = None;
        {
            recurse_html! { active_node root_nodes prev_tag_type $($remaining_html)* };
        }

        root_nodes.pop().unwrap()
    }};
}

#[macro_export]
macro_rules! recurse_html {
    // The beginning of an element without any attributes.
    // For <div></div> this is
    // <div>
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident < $start_tag:ident > $($remaining_html:tt)*) => {
        let mut new_node = VirtualNode::new(stringify!($start_tag));
        let mut new_node = Rc::new(RefCell::new(new_node));

        if ($prev_tag_type == None) {
            $root_nodes.push(Rc::clone(&new_node));
        } else {
            $active_node.as_mut().unwrap().borrow_mut().children.push(Rc::clone(&new_node));
            new_node.borrow_mut().parent = $active_node;
        }

        let mut $active_node = Some(new_node);

        let tag_type = Some(TagType::Open);
        recurse_html! { $active_node $root_nodes tag_type $($remaining_html)* }
    };

    // The beginning of an element.
    // For <div id="10",> this is
    // <div
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident < $start_tag:ident $($remaining_html:tt)*) => {
        let mut new_node = VirtualNode::new(stringify!($start_tag));
        let mut new_node = Rc::new(RefCell::new(new_node));

        if ($prev_tag_type == None) {
            $root_nodes.push(Rc::clone(&new_node));
        } else {
            $active_node.as_mut().unwrap().borrow_mut().children.push(Rc::clone(&new_node));
            new_node.borrow_mut().parent = $active_node;
        }

        $active_node = Some(new_node);

        let tag_type = Some(TagType::Open);
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
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident $prop_name:tt = $prop_value:expr, $($remaining_html:tt)*) => {
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


    // A closing tag for some associated opening tag name
    // For <div id="10",></div> this is:
    // </div>
    ($active_node:ident $root_nodes:ident $prev_tag_type:ident < / $end_tag:ident > $($remaining_html:tt)*) => {
        let tag_type = Some(TagType::Close);

        // TODO: Revisit this.. Feels like an unnecessary dance but idk
        let mut $active_node = Rc::clone(&$active_node.unwrap());
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

    #[test]
    fn empty_div() {
        let node = html!{
        <div></div>
        };

        let mut  expected_node = VirtualNode::new("div");

        assert_eq!(node, Rc::new(RefCell::new(expected_node)));
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

        assert_eq!(node, Rc::new(RefCell::new(expected_node)));
    }

    #[test]
    fn event() {
        struct TestStruct {
            closure_ran: bool
        };
        let mut test_struct = Rc::new(RefCell::new(TestStruct { closure_ran: false}));
        let mut test_struct_clone = Rc::clone(&test_struct);

        let node = html!{
        <div !onclick=move || {test_struct_clone.borrow_mut().closure_ran = true},></div>
        };


        assert_eq!(test_struct.borrow().closure_ran, false);

        node.borrow().events.0.get("onclick").unwrap()();

        assert_eq!(test_struct.borrow().closure_ran, true);
    }


    #[test]
    fn child_node() {
        let mut node = html!{
        <div><span></span></div>
        };

        let child = VirtualNode::new("span");
        let child = wrap(child);
        let mut child_clone = Rc::clone(&child);
        let mut children = vec![child];
        // TODO: Add parent

        let mut expected_node = VirtualNode::new("div");
        expected_node.children = children;
        let expected_node = wrap(expected_node);

        child_clone.borrow_mut().parent = Some(Rc::clone(&expected_node));

        assert_eq!(node, expected_node);
        assert_eq!(expected_node.borrow().children.len(), 1);
    }

    #[test]
    fn sibling_child_nodes() {
        let mut node = html!{
        <div><span></span><b></b></div>
        };

        let sibling1 = wrap(VirtualNode::new("span"));
        let sibling2 = wrap(VirtualNode::new("b"));

        let children = vec![sibling1, sibling2];

        let mut expected_node = VirtualNode::new("div");
        expected_node.children = children;
        let expected_node = wrap(expected_node);

        assert_eq!(node, expected_node);
        assert_eq!(node.borrow().children.len(), 2);

        for (index, child) in node.borrow().children.iter().enumerate() {
            assert_eq!(child, &expected_node.borrow().children[index]);
        }
    }

    fn wrap (v: VirtualNode) -> Rc<RefCell<VirtualNode>> {
        Rc::new(RefCell::new(v))
    }
}
