use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::Token;
use syn::parse::{Parse, ParseStream};

#[derive(Debug)]
pub(crate) struct TypeAndMaybeDollar {
    pub ty: syn::Type,
    /// Used to support `$crate::path::to::SomeType`.
    pub maybe_dollar: Option<syn::token::Dollar>,
}

#[derive(Debug)]
pub(crate) struct PathAndMaybeDollar {
    pub path: syn::Path,
    /// Used to support `$crate::path::to::SomeItem`.
    pub maybe_dollar: Option<syn::token::Dollar>,
}

impl Parse for TypeAndMaybeDollar {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Used to support `$crate::path::to::SomeType`.
        let maybe_dollar = input.parse::<Token![$]>().ok();

        let ty = input.parse::<syn::Type>()?;
        Ok(TypeAndMaybeDollar { ty, maybe_dollar })
    }
}

impl ToTokens for TypeAndMaybeDollar {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(dollar) = self.maybe_dollar.as_ref() {
            dollar.to_tokens(tokens);
        }
        self.ty.to_tokens(tokens)
    }
}

impl Parse for PathAndMaybeDollar {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // Used to support `$crate::path::to::SomeType`.
        let maybe_dollar = input.parse::<Token![$]>().ok();

        let path = input.parse::<syn::Path>()?;
        Ok(PathAndMaybeDollar { path, maybe_dollar })
    }
}

impl ToTokens for PathAndMaybeDollar {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(dollar) = self.maybe_dollar.as_ref() {
            dollar.to_tokens(tokens);
        }
        self.path.to_tokens(tokens)
    }
}
