//! Work in progress.. not finished..

extern crate proc_macro;
#[macro_use]
extern crate quote;
#[macro_use]
extern crate syn;
use proc_macro2::*;

use self::proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};
use syn::token::Token;

use std::collections::HashSet;

use syn::parse::{Parse, ParseStream, Result as SynResult, Error};
use syn::punctuated::Punctuated;
use syn::{Expr, Ident, ItemFn, Local, Pat, Stmt};
use quote::{ToTokens, TokenStreamExt};

/// Parsed attributes from a `#[route(..)]`.
#[derive(Default, Debug)]
struct RouteAttrs {
    attrs: Vec<RouteAttr>
}


impl Parse for RouteAttrs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        if input.is_empty() {
            return Ok(RouteAttrs { attrs: vec![]})
        }

        let opts = syn::punctuated::Punctuated::<_, syn::token::Comma>::parse_terminated(input)?;
        Ok(RouteAttrs {
            attrs: opts.into_iter().collect(),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RouteAttr {
    Path
}

impl Parse for RouteAttr {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let original = input.fork();
        let attr: Ident = input.parse()?;
        eprintln!("attr = {:#?}", attr);

        if attr == "path" {
            println!("WORKED");
            return Ok(RouteAttr::Path)
        }

        println!("NO WORK");

        Err(original.error("unknown attribute"))
    }
}

#[proc_macro_attribute]
pub fn route(args: TokenStream, input: TokenStream) -> TokenStream {
//    let input = parse_macro_input!(input as Token![struct]);


    let mut args = parse_macro_input!(args as RouteAttrs);

    return input.into();

    eprintln!("args = {:#?}", args);

    TokenStream::from(quote!(
    ))
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
