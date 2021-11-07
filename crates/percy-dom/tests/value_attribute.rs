//! Verify that when we set the value attribute for an input or textarea element we also call
//! .set_value(value) on the element. Without this the `.value()` method on the element won't
//! return the new value.

use wasm_bindgen_test::*;

use percy_dom::event::EventsByNodeIdx;
use wasm_bindgen::JsCast;
use web_sys::*;

use percy_dom::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Verify that we call .set_value when setting the value attribute on an input field.
///
/// Even if the value hasn't changed between diffs, we still overwrite it. This helps with making
/// sure that the value in the virtual dom overwrites anything that might have been typed into the
/// real DOM.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test value_attribute -- set_input_elem_value_property
#[wasm_bindgen_test]
fn set_input_elem_value_property() {
    for (start, end) in vec![("BOTH EQUAL", "BOTH EQUAL"), ("NOT", "EQUAL")] {
        let start_input = html! {<input value=start>};
        let end_input = html! {<input value=end>};

        let input_node = start_input.create_dom_node(0, &mut EventsByNodeIdx::new());

        input_node
            .clone()
            .dyn_into::<HtmlInputElement>()
            .unwrap()
            .set_value("Should Be Replaced");

        let patches = percy_dom::diff(&start_input, &end_input);

        percy_dom::patch(
            input_node.clone(),
            &end_input,
            &mut EventsByNodeIdx::new(),
            &patches,
        )
        .unwrap();

        assert_eq!(
            input_node.dyn_into::<HtmlInputElement>().unwrap().value(),
            end
        );
    }
}

/// Verify that we call .set_value when setting the value attribute on an textarea field.
///
/// Even if the value hasn't changed between diffs, we still overwrite it. This helps with making
/// sure that the value in the virtual dom overwrites anything that might have been typed into the
/// real DOM.
///
/// wasm-pack test --chrome --headless crates/percy-dom --test value_attribute -- set_textarea_elem_value_property
#[wasm_bindgen_test]
fn set_textarea_elem_value_property() {
    for (start, end) in vec![("BOTH EQUAL", "BOTH EQUAL"), ("NOT", "EQUAL")] {
        let start_textarea = html! {<textarea value=start>};
        let end_textarea = html! {<textarea value=end>};

        let textarea_node = start_textarea.create_dom_node(0, &mut EventsByNodeIdx::new());

        textarea_node
            .clone()
            .dyn_into::<HtmlTextAreaElement>()
            .unwrap()
            .set_value("Should Be Replaced");

        let patches = percy_dom::diff(&start_textarea, &end_textarea);

        percy_dom::patch(
            textarea_node.clone(),
            &end_textarea,
            &mut EventsByNodeIdx::new(),
            &patches,
        )
        .unwrap();

        assert_eq!(
            textarea_node
                .dyn_into::<HtmlTextAreaElement>()
                .unwrap()
                .value(),
            end
        );
    }
}
