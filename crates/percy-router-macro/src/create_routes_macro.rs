use quote::quote;
use syn;
use syn::parse::{Parse, ParseStream, Result as SynResult};
use syn::spanned::Spanned;
use syn::Ident;
use syn::{parse_macro_input, Path};

/// Creates Vec<RouteHandler> from a series of function names.
///
/// These functions should be annotated with the #[route(...)] macro,
/// since the #[route(...)] macro will generate the modules and route
/// handler creator functions that create_routes' generated code will use.
///
/// #### Macro
///
/// create_routes![a_route, another_route, self::path::to::route_fn]
///
/// #### Generated Code
///
/// vec![
///     __a_route_mod__::a_route_handler::new(),
///     __another_route_mod__::another_route_handler::new(),
///     self::path::to::__route_fn_mod__::route_fn_handler::new()
/// ]
pub fn create_routes(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let routes_to_create = parse_macro_input!(input as RoutesToCreate);

    let mut tokens = vec![];

    for route in routes_to_create.routes {
        let original_fn_name = route.segments.last().unwrap().into_tuple().0;
        let original_fn_name = original_fn_name.ident.to_string();

        let path_to_module_segments =
            &route.segments.iter().collect::<Vec<_>>()[0..route.segments.len() - 1];

        let route_mod = format!("__{}_mod__", original_fn_name);
        let route_mod = Ident::new(route_mod.as_str(), route.span());

        let route_fn = format!("{}_handler", original_fn_name);
        let route_fn = Ident::new(route_fn.as_str(), route.span());

        let path_tokens = if path_to_module_segments.len() > 0 {
            quote! {#(#path_to_module_segments)::* :: }
        } else {
            quote! {}
        };

        // path::to::module::__route_fn_name_mod__::route_fn_name_handler::new()
        let route_handler = quote! {
            std::rc::Rc::new( #path_tokens #route_mod :: #route_fn ::new())
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
    routes: Vec<Path>,
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
