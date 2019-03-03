#![recursion_limit = "128"]

extern crate proc_macro;

use quote::quote;
use syn;

use self::proc_macro::TokenStream;
use syn::parse_macro_input;

use syn::parse::{Parse, ParseStream, Result as SynResult};
use syn::FnArg;
use syn::ItemFn;
use syn::Pat;
use syn::{Ident, Lit, Token};
use syn::Type;

// FIXME: Get things working - then refactor and comment.

#[proc_macro_attribute]
pub fn route(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut args = parse_macro_input!(args as RouteAttrs);

    let original_fn = input.clone();

    let route_fn: RouteFn = parse_macro_input!(input as RouteFn);

    let route_fn_name = route_fn.route_fn.ident;

    let route_fn_mod = format!("__{}_mod__", route_fn_name);
    let route_fn_mod = Ident::new(&route_fn_mod, route_fn_name.span());

    let route_fn_handler = format!("{}_handler", route_fn_name);
    let route_fn_handler = Ident::new(&route_fn_handler, route_fn_name.span());

    let route_creator_fn = format!("create_{}", route_fn_name);
    let route_creator_fn = Ident::new(&route_creator_fn, route_fn_name.span());

    let params = route_fn.route_fn.decl.inputs;

    let route_handler_param_idents1: Vec<Ident> = params.clone()
        .into_iter()
        .map(|arg| {
            match arg {
                // some_param_name: type
                FnArg::Captured(captured) => match captured.pat {
                    Pat::Ident(pat) => pat.ident,
                    _ => unimplemented!("TODO: What should happen for other patterns?"),
                },
                _ => unimplemented!("TODO: What should happen for non captured args?"),
            }
        })
        .collect();
    let route_handler_param_idents2 = route_handler_param_idents1.clone();

    let route_handler_param_types: Vec<Type> = params.clone()
        .into_iter()
        .map(|arg| {
            match arg {
                // some_param_name: type
                FnArg::Captured(captured) => match captured.pat {
                    Pat::Ident(pat) => captured.ty,
                    _ => unimplemented!("TODO: What should happen for other patterns?"),
                },
                _ => unimplemented!("TODO: What should happen for non captured args?"),
            }
        })
        .collect();

    let route_handler_param_strings: Vec<String> = params
            .into_iter()
        .map(|arg| {
            match arg {
                // some_param_name: type
                FnArg::Captured(captured) => match captured.pat {
                    Pat::Ident(pat) => format!("{}", pat.ident),
                    _ => unimplemented!("TODO: What should happen for other patterns?"),
                },
                _ => unimplemented!("TODO: What should happen for non captured args?"),
            }
        })
        .collect();

    let mut tokens = vec![];

    // TODO: Don't force the path to be the first argument .. just getting tests passing ..
    if let RouteAttr::Path(ref path) = args.attrs[0] {
        // vec![":id", ":name", ...]
        let path_params: Vec<String> = match path {
            Lit::Str(path) => path
                .value()
                .split("/")
                .filter(|segment| segment.starts_with(":"))
                .map(|segment| without_first(segment).to_string())
                .collect(),
            _ => unimplemented!(""),
        };

        let route_creator = quote! {
            fn #route_creator_fn() -> Route {
                fn route_param_parser (param_key: &str, param_val: &str) -> Option<Box<dyn RouteParam>> {
                    // TODO: Generate this based on the attributes in the path and the arguments
                    // in the function.
                    match param_key {
                        "id" => {
                            return Some(Box::new(
                                u32::from_str_param(param_val).unwrap()
                            ));
                        }
                        _ => panic!("TODO: ")
                    };

                    // TODO: Generate a quote_spanned! error if we specify an attribute in the
                    // path that isn't in the arguments

                    None
                }

                Route::new(#path, Box::new(route_param_parser))
            }
        };

        tokens.push(route_creator);

        let route_handler = quote! {
            // Kept it it's own module so that we can enable non camel case types only
            // for this module. This way we don't need to worry as much about transforming
            // the generated struct name.
            pub mod #route_fn_mod {
                #[allow(non_camel_case_types)]

                use super::*;

                pub struct #route_fn_handler {
                    pub route: Route
                }

                impl #route_fn_handler {
                    pub fn new () -> #route_fn_handler {
                            #route_fn_handler {
                                route: #route_creator_fn()
                            }
                    }
                }

                impl RouteHandler for #route_fn_handler {
                    fn route (&self) -> &Route { &self.route }
                    fn view (&self, incoming_route: &str) -> VirtualNode {
                        // let id = self.route().find_route_param(incoming_route, "id").unwrap();
                        #(
                          let #route_handler_param_idents1 =
                           self.route().find_route_param(
                             incoming_route, #route_handler_param_strings
                           ).expect("Finding route param");
                        )*

                        #route_fn_name(
                          #(#route_handler_param_types::from_str_param(#route_handler_param_idents2).unwrap()),*
                        )
                    }
                }
            }
        };

        tokens.push(route_handler);
    }

    // Push the original function, without the #[route(...)] attribute now that we've
    // parsed it.
    tokens.push(original_fn.into());

    let tokens = quote! {
        #(#tokens)*
    };
    tokens.into()
}

#[proc_macro]
pub fn create_routes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut routes_to_create = parse_macro_input!(input as RoutesToCreate);

    let mut tokens = vec![];

    for route in routes_to_create.routes {
        let route_mod = format!("__{}_mod__", route);
        let route_mod = Ident::new(route_mod.as_str(), route.span());

        let route_fn = format!("{}_handler", route);
        let route_fn = Ident::new(route_fn.as_str(), route.span());

        // self::__route_fn_name_mod__::route_fn_name_handler::new()
        let route_handler = quote! {
            Box::new(self :: #route_mod :: #route_fn ::new())
        };
        tokens.push(route_handler);
    }

    let tokens = quote! {
        vec![#(#tokens),*]
    };

    tokens.into()
}

#[derive(Debug)]
struct RouteFn {
    route_fn: ItemFn,
}

impl Parse for RouteFn {
    fn parse(input: ParseStream) -> SynResult<Self> {
        Ok(RouteFn {
            route_fn: input.parse()?,
        })
    }
}

#[derive(Default, Debug)]
struct RoutesToCreate {
    routes: Vec<Ident>,
}

impl Parse for RoutesToCreate {
    fn parse(input: ParseStream) -> SynResult<Self> {
        if input.is_empty() {
            return Ok(RoutesToCreate { routes: vec![] });
        }

        let opts = syn::punctuated::Punctuated::<_, syn::token::Comma>::parse_terminated(input)?;

        Ok(RoutesToCreate {
            routes: opts.into_iter().collect(),
        })
    }
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

// https://www.reddit.com/r/rust/comments/8fpubp/how_to_remove_the_1st_character_from_str/dy5jdwk
fn without_first(string: &str) -> &str {
    string
        .char_indices()
        .next()
        .and_then(|(i, _)| string.get(i + 1..))
        .unwrap_or("")
}

