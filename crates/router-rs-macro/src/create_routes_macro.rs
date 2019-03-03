use quote::quote;
use syn;
use syn::parse::{Parse, ParseStream, Result as SynResult};
use syn::parse_macro_input;
use syn::{Ident, };

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
