#[feature(proc_macro)]
#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate lazy_static;
extern crate proc_macro;

use syn::Expr;

use proc_macro::TokenStream;
use proc_macro::TokenTree;

use std::env;
use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::sync::Mutex;

lazy_static! {
    static ref CSS_COUNTER: Mutex<u32> = Mutex::new(0);
}

/// Parses the syntax for writing inline css
///
/// css! {
///   "
///   :host {
///     background-color: red;
///   }
///
///   :host > div {
///     display: flex;
///     align-items: center;
///   }
///   "
/// }
#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    let mut css_counter = CSS_COUNTER.lock().unwrap();

    let class = format!("_iss_{}", css_counter);

    let css_file = env::vars().find(|(key, _)| key == "OUTPUT_CSS");

    if css_file.is_some() {
        let css_file = css_file.unwrap().1;
        eprintln!("css_file = {:#?}", css_file);

        if *css_counter == 0 {
            if Path::new(&css_file).exists() {
                fs::remove_file(&css_file);
            }

            let mut css_file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(css_file)
                .unwrap();

            for css in input.into_iter() {
                let css = css
                    .to_string()
                    .replace(":host", &format!(".{}", class))
                    .replace(r#"r#""#, "")
                    .replace("\"#", "")
                ;

                css_file.write(&css.into_bytes()).unwrap();
            }
        }
    }

    *css_counter += 1;

    let expanded = quote! {
    #class
    };

    expanded.into()
}
