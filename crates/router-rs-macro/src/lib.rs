extern crate proc_macro;

use quote::quote;
use syn;

use self::proc_macro::TokenStream;
use syn::parse_macro_input;

use syn::parse::{Parse, ParseStream, Result as SynResult};
use syn::{Ident, Token, Lit};

#[proc_macro_attribute]
pub fn route(args: proc_macro::TokenStream, input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut args = parse_macro_input!(args as RouteAttrs);

    let mut tokens = vec![];

    for attr in args.attrs {
        if let RouteAttr::Path(path) = attr {
            let route_handler = quote! {
                fn create_root_route() -> Route {
                    fn route_param_parser (param_key: &str, param_val: &str) -> Option<Box<dyn RouteParam>> {
                        // TODO: Generate this based on the attributes in the path and the arguments
                        // in the function.

                        // TODO: Generate a quote_spanned! error if we specify an attribute in the
                        // path that isn't in the arguments

                        None
                    }

                    Route::new("/", Box::new(route_param_parser))
                }
            };

            tokens.push(route_handler);
        }
    }

    let tokens = quote! {
        #(#tokens)*
    };
    tokens.into()
}

#[proc_macro]
pub fn create_routes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let tokens = quote! {
        vec![]
    };

    tokens.into()
}

/// Parsed attributes from a `#[route(..)]`.
#[derive(Default, Debug)]
struct RouteAttrs {
    attrs: Vec<RouteAttr>,
}

impl Parse for RouteAttrs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        if input.is_empty() {
            return Ok(RouteAttrs { attrs: vec![] });
        }

        let opts = syn::punctuated::Punctuated::<_, syn::token::Comma>::parse_terminated(input)?;

        Ok(RouteAttrs {
            attrs: opts.into_iter().collect(),
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
enum RouteAttr {
    Path(Lit),
}

impl Parse for RouteAttr {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let original = input.fork();

        // path = "/my/route/here"
        let path_key = input.parse::<Ident>()?;
        let equals = input.parse::<Token![=]>()?;
        let path_val = input.parse::<Lit>()?;

        if path_key == "path" {
            return Ok(RouteAttr::Path(path_val));
        }

        Err(original.error("unknown attribute"))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
