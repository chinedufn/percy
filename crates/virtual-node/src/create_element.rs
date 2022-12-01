use js_sys::Reflect;
use wasm_bindgen::JsValue;
use web_sys::{Document, Element};

use crate::event::{VirtualEventElement, VirtualEvents};
use crate::{AttributeValue, VElement, VirtualEventNode, VirtualNode};

mod add_events;

// Used to indicate that a DOM node was created from a virtual-node.
#[doc(hidden)]
pub const VIRTUAL_NODE_MARKER_PROPERTY: &'static str = "__v__";

impl VElement {
    /// Build a DOM element by recursively creating DOM nodes for this element and it's
    /// children, it's children's children, etc.
    pub(crate) fn create_element_node(
        &self,
        events: &mut VirtualEvents,
    ) -> (Element, VirtualEventElement) {
        let document = web_sys::window().unwrap().document().unwrap();

        let element = if html_validation::is_svg_namespace(&self.tag) {
            document
                .create_element_ns(Some("http://www.w3.org/2000/svg"), &self.tag)
                .unwrap()
        } else {
            document.create_element(&self.tag).unwrap()
        };
        set_virtual_node_marker(&element);

        self.attrs.iter().for_each(|(name, value)| {
            match value {
                AttributeValue::String(s) => {
                    element.set_attribute(name, s).unwrap();
                }
                AttributeValue::Bool(b) => {
                    if *b {
                        element.set_attribute(name, "").unwrap();
                    }
                }
            };
        });

        let events_id = events.unique_events_id();
        let mut event_elem = VirtualEventElement::new(events_id);
        self.add_events(&element, events, events_id);

        self.append_children_to_dom(&element, &document, &mut event_elem, events);

        self.special_attributes
            .maybe_call_on_create_element(&element);

        if let Some(inner_html) = &self.special_attributes.dangerous_inner_html {
            element.set_inner_html(inner_html);
        }

        (element, event_elem)
    }
}

impl VElement {
    fn append_children_to_dom(
        &self,
        element: &Element,
        document: &Document,
        event_node: &mut VirtualEventElement,
        events: &mut VirtualEvents,
    ) {
        let mut previous_node_was_text = false;

        self.children.iter().for_each(|child| {
            match child {
                VirtualNode::Text(text_node) => {
                    let current_node = element.as_ref() as &web_sys::Node;

                    // We ensure that the text siblings are patched by preventing the browser from merging
                    // neighboring text nodes. Originally inspired by some of React's work from 2016.
                    //  -> https://reactjs.org/blog/2016/04/07/react-v15.html#major-changes
                    //  -> https://github.com/facebook/react/pull/5753
                    //
                    // `ptns` = Percy text node separator
                    if previous_node_was_text {
                        let separator = document.create_comment("ptns");
                        set_virtual_node_marker(&separator);
                        current_node
                            .append_child(separator.as_ref() as &web_sys::Node)
                            .unwrap();
                    }

                    current_node
                        .append_child(&text_node.create_text_node())
                        .unwrap();
                    event_node.push_child(VirtualEventNode::Text);

                    previous_node_was_text = true;
                }
                VirtualNode::Element(element_node) => {
                    previous_node_was_text = false;

                    let (child, child_events) = element_node.create_element_node(events);
                    let child_elem: Element = child;

                    element.append_child(&child_elem).unwrap();
                    event_node.push_child(VirtualEventNode::Element(child_events))
                }
            }
        });
    }
}

/// Set a property on a node that can be used to know if a node was created by Percy.
pub(crate) fn set_virtual_node_marker(node: &JsValue) {
    let unused_data = 123;

    Reflect::set(
        &node.into(),
        &VIRTUAL_NODE_MARKER_PROPERTY.into(),
        &unused_data.into(),
    )
    .unwrap();
}
