//! # To Run
//!
//! cargo test -p html-macro-test -- ui trybuild=missing_closing_tag.rs

extern crate percy_dom;
use percy_dom::prelude::*;

// Open tag that are missing their corresponding closing tags.
fn main() {
    // One missing
    html! {
        <div>
    };

    // One missing with one valid
    html! {
        <strong>
          <span></span>
    };

    // Two missing
    html! {
        <em>
          <footer>
    };
}
