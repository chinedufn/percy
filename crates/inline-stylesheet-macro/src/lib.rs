#[feature(proc_macro)]

#[macro_use]
extern crate syn;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate lazy_static;
extern crate proc_macro;

use syn::Expr;
use syn::synom::Synom;

use proc_macro::TokenStream;
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
struct CSS {
    css: Expr
}

impl Synom for CSS {
    named!(parse -> Self, do_parse!(
        css: syn!(Expr) >>
        (CSS { css })
    ));
}

#[proc_macro]
pub fn css(input: TokenStream) -> TokenStream {
    let mut css_counter = CSS_COUNTER.lock().unwrap();

    let class = format!("_iss_{}", css_counter);

    let expanded = quote! {
    #class
    };

    *css_counter += 1;

    expanded.into()
}
