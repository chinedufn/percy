extern crate percy_vdom;
use percy_vdom::prelude::*;

// Used a tag name that does not exist in the HTML spec
fn main() {
    html! {
        <invalidtagname></invalidtagname>
    };
}
