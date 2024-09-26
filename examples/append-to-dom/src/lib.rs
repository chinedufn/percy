use percy_dom::{VirtualNode, JsCast, event::VirtualEvents, html};
use web_sys::{HtmlElement, HtmlInputElement};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
pub fn main() {
    // Create a div element that `percy` creates and updates.
    let div = html! { <div id="my_div" /> };

    // Append to DOM from `VirtualNode`.
    let mut events = VirtualEvents::new();
    let (div_node, enode) = div.create_dom_node(&mut events);
    events.set_root(enode);
    // Grab the div DOM element.
    let div_elem = div_node.dyn_ref::<HtmlElement>().unwrap();

    // Create a child checkbox node and append it to the DIV element.
    let document = web_sys::window().unwrap().document().unwrap();
    let default_checked_elem = document.create_element("input").unwrap();
    default_checked_elem.set_attribute("type", "checkbox").unwrap();
    default_checked_elem.set_attribute("checked", "").unwrap();
    // Add our `percy`-agnostic checkbox as a child of a `percy`-controlled DOM element.
    let child_elem = div_elem.append_child(&default_checked_elem).unwrap();
    
    // Create a access the child element and confirm it has the `checked` attribute.
    let child_checkbox_elem_ref = child_elem.dyn_ref::<HtmlInputElement>().unwrap();
    assert!(child_checkbox_elem_ref.has_attribute("checked"));
    // Modify the checkedness of the child element.
    // We will assert that it wasn't changed after a `percy-dom` update to ensure that `percy` isn't
    // modifying our appended element in any way.
    // Note that `percy` sets the `checked` attribute and property, whereas we're attempting to
    // maintain a checkbox with an independent `checked` attribute (which means that the default
    // checkedness is maintained: this might be useful for a form control that can be reset by the
    // `HTMLFormElement.reset()` method, for example - however this would be unusual for a `percy` app.)
    child_checkbox_elem_ref.set_checked(false);

    // Update the DOM according to the virtual node changes within the scope of `percy`.
    let updated_div = html! { <div id="my_div" data-custom="data" /> };
    let patches = percy_dom::diff(&div, &updated_div);
    percy_dom::patch(div_node.clone(), &updated_div, &mut events, &patches).unwrap();

    // Get a reference to the child of the div element and confirm that is maintains the
    // manually-specified attribute and property.
    let child_elem = div_elem.children().get_with_index(0).unwrap();
    let child_input_elem_ref = child_elem.dyn_ref::<HtmlInputElement>().unwrap();
    assert!(child_input_elem_ref.has_attribute("checked"));
    assert!(!child_input_elem_ref.checked());
}
