//!

use std::collections::HashMap;

#[derive(Debug)]
#[cfg_attr(test, derive(PartialEq))]
pub struct VirtualNode {
    tag: String,
    props: HashMap<String, String>,
    events: HashMap<String, fn() -> ()>,
    children: Vec<VirtualNode>,
}

/// A [Text node](https://developer.mozilla.org/en-US/docs/Web/API/Text).
/// When patching these into a real DOM these use `document.createTextNode(text)`
pub struct VirtualText(String);

pub trait VirtualElem {}

impl VirtualElem for VirtualNode {}

impl VirtualElem for VirtualText {}

pub fn createElement(node: impl VirtualElem) {
    // document.createElement(node.type)
}

// TODO: Move to html_macro.rs along w/ tests
#[macro_export]
macro_rules! html {
    ($($remaining_html:tt)*) => {{
        let mut parsed_html_stack = vec![];
        recurse_html! { parsed_html_stack ($($remaining_html)*) }
    }};
}

#[macro_export]
macro_rules! recurse_html {
    // The beginning of an element.
    // For <div id="10",> this is
    // <div
    ($parsed_html_stack:ident (< $start_tag:tt : $($remaining_html:tt)*)) => {
        println!("start of element");

        recurse_html! { $parsed_html_stack ($($remaining_html)*) }
    };

    // The end of an opening tag.
    // For <div id="10",> this is:
    //  >
    ($parsed_html_stack:ident (> $($remaining_html:tt)*)) => {
        println!("opening tag");

        recurse_html! { $parsed_html_stack ($($remaining_html)*) }
    };

    // A property
    // For <div id="10",> this is:
    // id = "10"
    ($parsed_html_stack:ident ($prop_name:tt = $prop_value:expr, $($remaining_html:tt)*)) => {
        println!("identifier");

        recurse_html! { $parsed_html_stack ($($remaining_html)*) }
    };

    // A closing tag for some associated opening tag name
    // For <div id="10",></div> this is:
    // </div>
    ($parsed_html_stack:ident (< / $end_tag:ident > $($remaining_html:tt)*)) => {
        println!("End of associated tag");
        recurse_html! { $parsed_html_stack ($($remaining_html)*) }
    };

    // Done parsing some element's closing tag
    ($parsed_html_stack:ident ()) => {
        println!("foo bar");
    };

    // TODO: README explains that props must end with commas
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_div() {
        let node = html!{
        i="hello world",
        };
//        <div></div>

        let expected_node = VirtualNode {
            tag: "div".to_string(),
            props: HashMap::new(),
            events: HashMap::new(),
            children: vec![],
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
            events: HashMap::new(),
            children: vec![],
        };

        assert_eq!(node, expected_node);
    }
}
