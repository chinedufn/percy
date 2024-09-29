//! `percy-dom` treats the `checked` virtual node attribute specially.
//! `percy-dom` sets the `checked` element property (actual checkedness),
//! not the `checked` HTML attribute (default checkedness).
//!
//! Developers, are likely to assume that `checked` specifies the state of the checkbox
//! directly. `percy-dom` ensures that this is true.
//!
//! See the tests for more details. Start with [`patch_uses_set_checked_function`].
//!
//! To run all tests in this file:
//!
//! ```sh
//! wasm-pack test --chrome --headless crates/percy-dom --test checked_property
//! ```

use wasm_bindgen_test::*;

use percy_dom::event::VirtualEvents;
use wasm_bindgen::JsCast;
use web_sys::*;

use percy_dom::prelude::*;

wasm_bindgen_test_configure!(run_in_browser);

/// Verify that `percy_dom::patch` uses `set_checked` to set the checkedness
/// of an input element when specified.
///
/// ## Why?
///
/// The `checked` HTML attribute only determines the default checkedness.
/// The browser uses the default checkedness as the checkbox's
/// checkedness until the user clicks on the checkbox, setting the "dirty checked" browser
/// flag https://html.spec.whatwg.org/multipage/input.html#concept-input-checked-dirty
/// which results in the browser maintaining the checkbox's state INDEPENDENTLY from the
/// default checked state. i.e. Changing the default checkedness no longer affects the actual
/// checkedness after the user has pressed the input.
///
/// We want `html!{ ... checked=val ... }` to specify the current checkedness of the checkbox
/// directly - avoiding the checkbox being toggled differently to what the developer
/// specified in the virtual DOM.
///
/// Using web-sys's `set_checked` sets the actual checkbox's checkedness. Futhermore, it enables the
/// dirty-checked flag (NB: BUT ONLY WHEN THE CHECKBOX STATE IS CHANGED), which we can test for.
///
/// ## Test approach
///
/// - Create a virtual node with the checkbox having checkedness !C, and patch it to have checkedness C.
///     (This should cause the dirty flag to be set IF `set_checked` is used.)
/// - Assert that the corresponding DOM element has checkedness of C.
///
/// - Now, remove the attribute if the checkbox is checked, or set the attribute if not.
///     (The checkbox should hold its state as the dirty flag is checked, therefore
///     changing the default checkedness through the `checked` attribute no longer
///     should affect the checkedness of the checkbox.)
/// - Assert that the checkedness of the checkbox element is still B.
#[wasm_bindgen_test]
fn patch_sets_checked_property() {
    for checkedness in [false, true] {
        let start_input = html! {<input checked={!checkedness}>};
        let end_input = html! {<input checked=checkedness>};

        let mut events = VirtualEvents::new();
        let (input_node, enode) = start_input.create_dom_node(&mut events);
        events.set_root(enode);

        let input_elem = input_node.dyn_ref::<HtmlInputElement>().unwrap();

        let patches = percy_dom::diff(&start_input, &end_input);
        percy_dom::patch(input_node.clone(), &end_input, &mut events, &patches).unwrap();
        assert_eq!(input_elem.checked(), checkedness);

        if checkedness {
            input_elem.remove_attribute("checked").unwrap();
        } else {
            input_elem.set_attribute("checked", "").unwrap();
        }
        assert_eq!(input_elem.checked(), checkedness);
    }
}

/// Verify that `percy_dom::patch` uses `set_checked` to set the `checked` property
/// of an input element even if the the specified `checked` value does not change
/// between the `old` and `new` virtual nodes.
///
/// ## Why?
///
/// Note: the rationale given in [`patch_sets_checked_property`] is prerequisite reading.
///
/// The user might interact with the checkbox in between the previous render and the next
/// one, changing the checkedness state in the browser, but `diff` would not realize this
/// assuming the rendered `checked` value does not change.
///
/// For example:
/// - Developer renders `html! { ... checked=true ... }`
/// - User clicks on the checkbox, changing the browser's checkbox checkedness to false.
/// - Developer renders `html! { ... checked=true ... }`
/// - `diff` doesn't realize anything needs to change, so it doesn't issue any changes.
/// - Developer is still trying to render the checkbox as checked but the browser checkbox
///     stays unchecked.
///
/// If `percy_dom::diff` always specifies that `percy_dom::patch` should set the `checked`
/// property if its specified, then the above cannot happen. The element's checked state
/// will be fixed when `percy_dom::patch` is called, keeping the developer-specified `checked`
/// value and the checkbox element's visual state in sync.
///
/// ## Test approach
///
/// - Create a a DOM node with the checkbox having checkedness C.
/// - Set it's checkedness to be !C.
/// - Diff and patch with the virtual node still specifying it's checkedness as C.
/// - Assert that the checkedness has been reset to C, even though the virtual node did not change.
#[wasm_bindgen_test]
fn patch_always_sets_checked_property() {
    for checkedness in [false, true] {
        let start_input = html! {<input checked=checkedness>};
        let end_input = html! {<input checked=checkedness>};

        let mut events = VirtualEvents::new();
        let (input_node, enode) = start_input.create_dom_node(&mut events);
        events.set_root(enode);

        let input_elem = input_node.dyn_ref::<HtmlInputElement>().unwrap();
        assert_eq!(input_elem.checked(), checkedness);

        input_elem.set_checked(!checkedness); // modify checked property

        let patches = percy_dom::diff(&start_input, &end_input);
        percy_dom::patch(input_node.clone(), &end_input, &mut events, &patches).unwrap();

        assert_eq!(input_elem.checked(), checkedness);
    }
}

/// Verify that `percy_dom::patch` does not set the default checkedness.
/// That is to say: it does NOT manipulate the HTML `checked` attribute.
///
/// ## Why?
///
/// Note: the rationale given in [`patch_sets_checked_property`] is prerequisite reading.
///
/// `percy` is intended to serve the developer who is making a nontrivial app who's state
/// is driven by the backend of the application. The application state is _not_ driven by
/// the frontend. Therefore the state of checkboxes is specified explicitly idiomatically.
///
/// `percy-dom` does not currently allow for a way to directly manipulate the default
/// checkedness of an input element. Default checkedness is not useful unless the state
/// of an input element is driven by the frontend.
///
/// Users may want to break out of this mold however, and modify the default checkedness.
/// In this case, it's much simpler if `percy` doesn't change the default checkedness
/// unnecessarily. Equivalently, `percy-dom` does not manipulate the presence of the HTML
/// `checked` attribute.
///
/// ## Test approach
///
/// - Create an input element with checkedness C.
/// - Set the presence of the HTML attribute (default checkedness) to be D.
/// - Update the input element's checkedness to be E.
/// - Assert that the default checkedness is still D.
#[wasm_bindgen_test]
fn patch_does_not_set_default_checkedness() {
    for prev_checkedness in [false, true] {
        for next_checkedness in [false, true] {
            for default_checkedness in [false, true] {
                let start_input = html! {<input checked=prev_checkedness>};
                let end_input = html! {<input checked=next_checkedness>};

                let mut events = VirtualEvents::new();
                let (input_node, enode) = start_input.create_dom_node(&mut events);
                events.set_root(enode);

                let input_elem = input_node.dyn_ref::<HtmlInputElement>().unwrap();
                assert_eq!(input_elem.checked(), prev_checkedness);

                // Modify presence of `checked`` attribute.
                input_elem.set_default_checked(default_checkedness);

                let patches = percy_dom::diff(&start_input, &end_input);
                percy_dom::patch(input_node.clone(), &end_input, &mut events, &patches).unwrap();

                assert_eq!(input_elem.default_checked(), default_checkedness);
            }
        }
    }
}
