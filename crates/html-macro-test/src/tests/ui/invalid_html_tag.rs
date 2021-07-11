extern crate percy_dom;
use percy_dom::prelude::*;

// Used a tag name that does not exist in the HTML spec
fn main() {
    html! {
        <invalidtagname></invalidtagname>
    };
}
