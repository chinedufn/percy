use percy_dom::{event::VirtualEvents, prelude::*, JsCast};
use web_sys::{Element, Node};

use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Verify that the
/// - `percy-dom`-controlled element, `root`,
/// - and the foreign DOM element, `my_special_paragraph`
///
/// behave exactly as we expect when updating them...
///
/// - Updating the `percy`-controlled element `root` does not affect the foreign elements.
/// - The state of the foreign elements are preserved across `percy_dom::patch` calls.
#[wasm_bindgen_test]
fn checkbox_checkedness_update_test() {
    let (vdom_root, dom_root, mut events, my_special_paragraph) =
        setup_percy_dom_with_embedded_element();

    // Modify the `my_special_paragraph` element
    my_special_paragraph.set_text_content(Some("world"));

    // Adds the "data-example" attribute, set to "New data!", to the root node.
    percy_diff_and_patch_root_node(&vdom_root, &dom_root, &mut events);

    // Assert that our modification of the DOM element was successful.
    // Assert that `percy-dom` didn't overwrite our changes.
    assert_eq!(
        my_special_paragraph.text_content(),
        Some("world".to_string())
    );

    // Assert that `percy-dom`'s diff+patch succeeded.
    let data_example = dom_root
        .dyn_ref::<Element>()
        .unwrap()
        .get_attribute("data-example")
        .unwrap();
    assert_eq!(&data_example, "New data!");
}

fn create_my_special_paragraph_element() -> Element {
    let document = web_sys::window().unwrap().document().unwrap();

    let element = document.create_element("p").unwrap();
    element.set_id("my_special_paragraph");
    element.set_text_content(Some("hello"));

    element
}

fn setup_percy_dom_with_embedded_element() -> (VirtualNode, Node, VirtualEvents, Element) {
    let my_div_element = create_my_special_paragraph_element();
    let my_div_element_append = my_div_element.clone();

    // The `div` element will be the child of the percy-controlled root element.
    let vdom_root_node = html! {
        <div
            id="root"
            key="key"
            on_create_element=move |elem| { elem.append_child(&my_div_element_append).unwrap(); }
        />
    };

    // Create the DOM nodes from the virtual DOM.
    let mut events = VirtualEvents::new();
    let (root_node, event_node) = vdom_root_node.create_dom_node(&mut events);
    events.set_root(event_node);

    (vdom_root_node, root_node, events, my_div_element)
}

fn percy_diff_and_patch_root_node(
    vdom_root: &VirtualNode,
    dom_root: &Node,
    events: &mut VirtualEvents,
) {
    let new_vdom_root = html! { <div id="root" data-example="New data!" /> };
    let patches = percy_dom::diff(&vdom_root, &new_vdom_root);
    percy_dom::patch(dom_root.clone(), &new_vdom_root, events, &patches).unwrap();
}
