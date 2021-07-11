extern crate percy_vdom;
use percy_vdom::prelude::*;

// Expected a closing div tag, found a closing strong tag
fn main() {
    html! {
        <div> </strong>
    };
}
