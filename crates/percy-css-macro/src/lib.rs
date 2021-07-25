//! percy-css-macro is a procedural macro that allows you to write your CSS next to your Rust views.
//!
//! github.com/chinedufn/percy/examples/css-in-rust

#[macro_use]
extern crate quote;
#[macro_use]
extern crate lazy_static;
extern crate proc_macro;

use proc_macro::TokenStream;

use std::env;
use std::fs;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

lazy_static! {
    static ref CSS_COUNTER: Mutex<u32> = Mutex::new(0);
}

/// Parses the syntax for writing inline css. Every call to css! will have its class
/// name incremented by one.
///
/// So your first css! call is class "._css_rs_0", then "._css_rs_1", etc.
///
/// To write your css to a file use:
///
/// ```ignore
/// OUTPUT_CSS=/path/to/my/output.css cargo run my-app
/// ```
///
/// # Examples
///
/// ```ignore
/// #[macro_use]
/// extern crate percy_css_macro;
///
/// fn main () {
///     let class1 = css! {
///       "
///       :host {
///         background-color: red;
///       }
///
///       :host > div {
///         display: flex;
///         align-items: center;
///       }
///       "
///     };
///
///     let class2 = css! {r#"
///         :host { display: flex; }
///     "#};
///
///     assert_eq!(class1, "_css_rs_0".to_string());
///     assert_eq!(class2, "_css_rs_1".to_string());
/// }
/// ```
#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    let mut css_counter = CSS_COUNTER.lock().unwrap();

    let class = format!("_css_rs_{}", css_counter);

    let css_file = env::vars().find(|(key, _)| key == "OUTPUT_CSS");

    if css_file.is_some() {
        let css_file = css_file.unwrap().1;

        if *css_counter == 0 {
            if Path::new(&css_file).exists() {
                fs::remove_file(&css_file).unwrap();
            }

            let mut css_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(css_file)
                .unwrap();

            write_css_to_file(&mut css_file, &class, input);
        } else {
            let mut css_file = OpenOptions::new().append(true).open(css_file).unwrap();

            write_css_to_file(&mut css_file, &class, input);
        }
    }

    *css_counter += 1;

    let expanded = quote! {
    #class
    };

    expanded.into()
}

fn write_css_to_file(css_file: &mut File, class: &str, input: TokenStream) {
    for css in input.into_iter() {
        let mut css = css.to_string();

        // Remove the surrounding quotes from so that we can write only the
        // CSS to our file.
        //
        // Handles:
        //   css!{r#" :host { ... } "#}
        //     as well as
        //   css!{" :host { ... } "}
        let first_quote_mark = css.find(r#"""#).unwrap();
        let last_quote_mark = css.rfind(r#"""#).unwrap();
        css.truncate(last_quote_mark);
        let mut css = css.split_off(first_quote_mark + 1);

        // Replace :host selectors with the class name of the :host element
        // A fake shadow-dom implementation.. if you will..
        let css = css.replace(":host", &format!(".{}", class));

        css_file.write(&css.into_bytes()).unwrap();
        css_file.write("\n".as_bytes()).unwrap();
    }
}
