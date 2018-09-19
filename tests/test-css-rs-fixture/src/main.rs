#![feature(proc_macro_non_items)]

extern crate css_rs_macro;

use css_rs_macro::css;

fn main() {
    let class1 = css! {r#"
    :host {
     color: red;
     background-color: blue;
    }
    "#};

    let class2 = css!{"
    :host {
        display: flex;
    }
    "};

    // We need to use both of these variables so that the compiler doesn't optimize them away.
    // If we didn't do this our test-css-rs integration test that builds this crate a would not
    // be able to test that our CSS was written to disk.
    assert_ne!(class1, class2);
}
