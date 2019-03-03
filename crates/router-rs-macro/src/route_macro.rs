use proc_macro2::Span;
use quote::quote;
use std::collections::HashSet;
use syn;
use syn::parse::{Parse, ParseStream, Result as SynResult};
use syn::parse_macro_input;
use syn::punctuated::Punctuated;
use syn::FnArg;
use syn::ItemFn;
use syn::Pat;
use syn::Type;
use syn::{Ident, Lit, Token};
use std::collections::HashMap;

// FIXME: Clean up to use consistent temrminology:
// route_def_colon_param, route_fn_param_ident, route_fn_param_ty

pub fn route(
    args: proc_macro::TokenStream,
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut args = parse_macro_input!(args as RouteAttrs);

    let original_fn = input.clone();

    let route_fn: RouteFn = parse_macro_input!(input as RouteFn);

    let mut tokens = vec![];

    // Push the original function, without the #[route(...)] attribute now that we've
    // parsed it.
    tokens.push(original_fn.into());

    let route_fn_name = route_fn.route_fn.ident;

    let create_route = format!("create_{}", route_fn_name);
    let create_route = Ident::new(&create_route, route_fn_name.span());

    let params = route_fn.route_fn.decl.inputs;

    let types = as_param_types(&params);

    let mut type_indices = HashMap::new();
    for (idx, param) in as_param_idents(&params).iter().enumerate() {
        type_indices.insert(format!("{}", param), idx);
    }

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
        let path_params2 = path_params
            .clone()
            .into_iter()
            .map(|ident| Ident::new(&ident, Span::call_site()))
            .collect();
        let path_params_map: HashSet<String> = path_params.clone().into_iter().collect();

        let mut path_param_types = vec![];
        for path_param in path_params.iter() {
            let type_idx = type_indices.get(path_param).unwrap();
            path_param_types.push(types[*type_idx]);
        }

        let route_creator = quote! {
            fn #create_route() -> Route {
                fn route_param_parser (param_key: &str, param_val: &str) -> Option<Box<dyn RouteParam>> {
                    // TODO: Generate this based on the attributes in the path and the arguments
                    // in the function.
                    match param_key {
                        #(
                            #path_params => {
                                return Some(Box::new(
                                    #path_param_types::from_str_param(param_val).expect("Macro parsed param")
                                ));
                            }
                        )*
                        _ => panic!("TODO: Think about when this case gets hit... 2am coding ...")
                    };

                    // TODO: Generate a quote_spanned! error if we specify an attribute in the
                    // path that isn't in the arguments

                    None
                }

                Route::new(#path, Box::new(route_param_parser))
            }
        };

        tokens.push(route_creator);

        let route_handler_mod =
            gen_route_handler_mod(route_fn_name, &params, &path_params_map, path_params2);
        tokens.push(route_handler_mod);
    }

    let tokens = quote! {
        #(#tokens)*
    };
    tokens.into()
}

fn gen_route_handler_mod(
    route_fn_name: Ident,
    params: &Punctuated<FnArg, Token![,]>,
    path_params_map: &HashSet<String>,
    path_params2: Vec<Ident>,
) -> proc_macro2::TokenStream {
    let route_fn_mod = format!("__{}_mod__", route_fn_name);
    let route_fn_mod = Ident::new(&route_fn_mod, route_fn_name.span());

    let route_fn_handler = format!("{}_handler", route_fn_name);
    let route_fn_handler = Ident::new(&route_fn_handler, route_fn_name.span());

    let create_route = format!("create_{}", route_fn_name);
    let create_route = Ident::new(&create_route, route_fn_name.span());

    let param_idents = as_param_idents(params);
    let param_idents2 = as_param_idents(params);

    let param_ident_strings: Vec<String> = as_param_idents(params)
        .iter()
        .filter(|param| path_params_map.contains(&format!("{}", param)))
        .map(|ident| format!("{}", ident))
        .collect();

    let types = as_param_types(&params);

    // let state: Provided<State> = ... ;
    // let more_data: Provided<Foo> = ...;
    let mut argument_definitions = vec![];

    let mut arguments = vec![];

    for (idx, arg_type) in types.iter().enumerate() {
        let param = param_idents[idx];

        if path_params_map.contains(&format!("{}", param)) {
            arguments.push(quote! {
                #arg_type::from_str_param(#param).expect(&format!("Error parsing param {}", #param))
            });
            continue;
        }

        // This is not a route parameter, so it must be a provided parameter

        argument_definitions.push(quote! {
        let #param = self
            .provided()
            .borrow();
        let #param = #param
            .get(&std::any::TypeId::of::<#arg_type>())
            .unwrap()
            .downcast_ref::<#arg_type>()
            .expect("Downcast param");
        });

        arguments.push(quote! {
            Provided::clone(#param)
        });

        // TODO: If this isn't a provided parameter or a route param.
        // Generate a compiler error. Test this with our ui crate.
    }

    let route_handler_mod = quote! {
        // Kept it it's own module so that we can enable non camel case types only
        // for this module. This way we don't need to worry as much about transforming
        // the generated struct name.
        pub mod #route_fn_mod {
            #[allow(non_camel_case_types)]

            use super::*;

            pub struct #route_fn_handler {
                route: Route,
                provided: Option<ProvidedMap>
            }

            impl #route_fn_handler {
                pub fn new () -> #route_fn_handler {
                        #route_fn_handler {
                            route: #create_route(),
                            provided: None
                        }
                }
            }

            impl RouteHandler for #route_fn_handler {
                fn route (&self) -> &Route { &self.route }

                fn set_provided (&mut self, provided: ProvidedMap) {
                    self.provided = Some(provided);
                }

                fn provided (&self) -> &ProvidedMap {
                    &self.provided.as_ref().unwrap()
                }

                fn view (&self, incoming_route: &str) -> VirtualNode {
                    // example:
                    //   let id = self.route().find_route_param(incoming_route, "id").unwrap();
                    #(
                      let #path_params2 =
                       self.route().find_route_param(
                         incoming_route, #param_ident_strings
                       ).expect("Finding route param");
                    )*

                    #(#argument_definitions)*

                    #route_fn_name(
                        #( #arguments ),*
                    )
                }
            }
        }
    };

    route_handler_mod
}

fn as_param_idents(params: &Punctuated<FnArg, Token![,]>) -> Vec<&Ident> {
    params
        .iter()
        .map(|arg| {
            match arg {
                // some_param_name: type
                FnArg::Captured(captured) => match captured.pat {
                    Pat::Ident(ref pat) => &pat.ident,
                    _ => unimplemented!("TODO: What should happen for other patterns?"),
                },
                _ => unimplemented!("TODO: What should happen for non captured args?"),
            }
        })
        .collect()
}

fn as_param_types(params: &Punctuated<FnArg, Token![,]>) -> Vec<&Type> {
    params
        .iter()
        .map(|arg| {
            match arg {
                // some_param_name: type
                FnArg::Captured(captured) => match captured.pat {
                    Pat::Ident(ref pat) => &captured.ty,
                    _ => unimplemented!("TODO: What should happen for other patterns?"),
                },
                _ => unimplemented!("TODO: What should happen for non captured args?"),
            }
        })
        .collect()
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

fn without_first(string: &str) -> &str {
    string
        .char_indices()
        .next()
        .and_then(|(i, _)| string.get(i + 1..))
        .unwrap_or("")
}
