extern crate percy_dom;
use percy_dom::prelude::*;

// Expected a closing div tag, found a closing strong tag
fn main() {
    html! {
        <div> </strong>
    };
}
