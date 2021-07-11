extern crate percy_vdom;
use percy_vdom::prelude::*;

// We are using open and close tags for a tag that should
// actually be a self closing tag
fn main() {
    html! {
        <br></br>
    };
}
