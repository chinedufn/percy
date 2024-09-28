use percy_dom::{event::VirtualEvents, html, IterableNodes, JsCast, VElement, VirtualNode};
use wasm_bindgen_test::*;
use web_sys::{HtmlInputElement, Node};

wasm_bindgen_test_configure!(run_in_browser);

fn create_my_default_checked_checkbox() -> HtmlInputElement {
    let document = web_sys::window().unwrap().document().unwrap();

    let checkbox = document
        .create_element("input")
        .unwrap()
        .dyn_into::<HtmlInputElement>()
        .unwrap();
    checkbox.set_id("my_default_checked_checkbox");
    checkbox.set_type("checkbox");
    checkbox.set_default_checked(true);
    checkbox.set_checked(true);

    checkbox
}

fn setup_percy_dom_with_appended_child_checkbox(
) -> (VirtualNode, Node, VirtualEvents, HtmlInputElement) {
    // This will be a checkbox that `percy-dom` controls.
    let vdom_percy_checkbox = html! {
        <input id="percy_checkbox" type="checkbox" checked=true>
    };

    let my_default_checked_checkbox = create_my_default_checked_checkbox();
    let my_default_checked_checkbox_append = my_default_checked_checkbox.clone();

    // Manually create a container VirtualNode that will append my-default-checked-checkbox
    // onto its corresponding DOM element upon creation.
    let mut vdom_checkbox_holder = VElement::new("div");
    vdom_checkbox_holder
        .attrs
        .insert("id".into(), "checkbox_holder".into());
    vdom_checkbox_holder.children.push(vdom_percy_checkbox);
    vdom_checkbox_holder
        .special_attributes
        .set_on_create_element("key", move |e| {
            e.append_child(&my_default_checked_checkbox_append).unwrap();
        });

    // Parent the container to some other node for the sake of example.
    let vdom_root_node = html! {
        <div id="root"> { VirtualNode::Element(vdom_checkbox_holder) } </div>
    };

    // Create the DOM nodes from the virtual DOM.
    let mut events = VirtualEvents::new();
    let (root_node, event_node) = vdom_root_node.create_dom_node(&mut events);
    events.set_root(event_node);

    (
        vdom_root_node,
        root_node,
        events,
        my_default_checked_checkbox,
    )
}

fn uncheck_both_checkboxes(
    vdom_root: &VirtualNode,
    dom_root: Node,
    events: &mut VirtualEvents,
    my_default_checked_checkbox: HtmlInputElement,
) {
    // Update the default-checked checkbox we maintain to be unchecked
    my_default_checked_checkbox.set_checked(false);

    // Update the percy checkbox to unchecked
    let new_vdom_root = html! {
        <div id="root">
            <div id="checkbox_holder">
                <input id="percy_checkbox" type="checkbox" checked=false>
            </div>
        </div>
    };
    let patches = percy_dom::diff(&vdom_root, &new_vdom_root);
    percy_dom::patch(dom_root.clone(), &new_vdom_root, events, &patches).unwrap();
}

/// Verify that the `percy-dom`-controlled checkbox, `percy_checkbox`, and the
/// `my_default_checked_checkbox` behave exactly as we expect when updating the checkedness:
/// - Both `percy_checkbox`'s checkedness and default checkedness should change.
/// - Only `my_default_checked_checkbox`'s checkedness should change, not default checkedness.
#[wasm_bindgen_test]
fn checkbox_checkedness_update_test() {
    // Setup both checkboxes and create the DOM node tree.
    let (vdom_root, dom_root, mut events, my_checkbox) =
        setup_percy_dom_with_appended_child_checkbox();

    // This allows us to use `document.get_element_by_id("...")` which is convenient for testing purposes.
    // This is NOT necessary.
    web_sys::window()
        .unwrap()
        .document()
        .unwrap()
        .body()
        .unwrap()
        .append_child(&dom_root)
        .unwrap();

    ensure_percy_and_default_checked_checkboxes_are_checked();

    uncheck_both_checkboxes(&vdom_root, dom_root, &mut events, my_checkbox);

    // Both checkboxes should now be unchecked.
    ensure_checkboxes_are_unchecked();

    // `percy` should have overridden the default checkedness to match the checkedness.
    ensure_percy_checkbox_is_no_longer_default_checked();

    // My default-checked checkbox should still be checked by default,
    // as we didn't override default checkedness ourselves.
    ensure_my_default_checked_checkbox_is_still_default_checked();
}

fn ensure_percy_and_default_checked_checkboxes_are_checked() {
    let document = web_sys::window().unwrap().document().unwrap();

    let percy_checkbox = document.get_element_by_id("percy_checkbox").unwrap();
    let percy_checkbox_ref = percy_checkbox.dyn_ref::<HtmlInputElement>().unwrap();
    assert!(percy_checkbox_ref.checked());

    let default_checked_checkbox = document
        .get_element_by_id("my_default_checked_checkbox")
        .unwrap();
    let default_checked_checkbox_ref = default_checked_checkbox
        .dyn_ref::<HtmlInputElement>()
        .unwrap();
    assert!(default_checked_checkbox_ref.checked());
}

fn ensure_checkboxes_are_unchecked() {
    let document = web_sys::window().unwrap().document().unwrap();

    let percy_checkbox = document.get_element_by_id("percy_checkbox").unwrap();
    let percy_checkbox_ref = percy_checkbox.dyn_ref::<HtmlInputElement>().unwrap();
    assert!(!percy_checkbox_ref.checked());

    let default_checked_checkbox = document
        .get_element_by_id("my_default_checked_checkbox")
        .unwrap();
    let default_checked_checkbox_ref = default_checked_checkbox
        .dyn_ref::<HtmlInputElement>()
        .unwrap();
    assert!(!default_checked_checkbox_ref.checked());
}

fn ensure_percy_checkbox_is_no_longer_default_checked() {
    let document = web_sys::window().unwrap().document().unwrap();

    let percy_checkbox = document.get_element_by_id("percy_checkbox").unwrap();
    let percy_checkbox_ref = percy_checkbox.dyn_ref::<HtmlInputElement>().unwrap();
    assert!(!percy_checkbox_ref.default_checked());
}

fn ensure_my_default_checked_checkbox_is_still_default_checked() {
    let document = web_sys::window().unwrap().document().unwrap();

    let default_checked_checkbox = document
        .get_element_by_id("my_default_checked_checkbox")
        .unwrap();
    let default_checked_checkbox_ref = default_checked_checkbox
        .dyn_ref::<HtmlInputElement>()
        .unwrap();
    assert!(default_checked_checkbox_ref.default_checked());
}
