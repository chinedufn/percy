#![feature(use_extern_macros)]
#![feature(proc_macro_non_items)]

extern crate inline_stylesheet_macro;

use inline_stylesheet_macro::css;

fn main() {
    let css_file = ::std::env::vars().find(|(key, _)| key == "OUTPUT_CSS");
    let css_file = css_file.unwrap();

    css! {r#"
    :host {
     color: red;
     background-color: blue;
    }
    "#};

    css!{"
    :host {
        display: flex;
    }
    "};
}
