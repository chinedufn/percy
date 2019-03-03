#![recursion_limit = "128"]

extern crate proc_macro;

use self::proc_macro::TokenStream;
use syn;

mod create_routes_macro;
mod route_macro;

// FIXME: Get things working - then refactor and comment.

// FIXME: Deny missing docs

#[proc_macro_attribute]
pub fn route(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    route_macro::route(args, input)
}

#[proc_macro]
pub fn create_routes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    create_routes_macro::create_routes(input)
}

