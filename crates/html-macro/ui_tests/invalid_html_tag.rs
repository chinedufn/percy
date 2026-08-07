//! # To Run
//!
//! cargo test -p html-macro-test --lib ui -- trybuild=invalid_html_tag.rs

use percy_dom::html;
use percy_dom::prelude::*;

// Used a tag name that does not exist in the HTML spec
fn main() {
    html! {
        <invalidtagname></invalidtagname>
    };
}
