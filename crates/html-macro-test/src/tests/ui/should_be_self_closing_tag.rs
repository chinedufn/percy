extern crate percy_dom;
use percy_dom::prelude::*;

// We are using open and close tags for a tag that should
// actually be a self closing tag
fn main() {
    html! {
        <br></br>
    };
}
