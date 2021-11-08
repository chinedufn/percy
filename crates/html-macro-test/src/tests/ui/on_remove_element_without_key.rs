extern crate percy_dom;
use percy_dom::prelude::*;

// Used the `on_remove_element` attribute without providing a key attribute.
fn main() {
    html! {
        <div on_remove_element = ||{} >
        </div>
    };
}
