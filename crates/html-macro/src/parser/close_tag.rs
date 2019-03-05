use crate::parser::{is_self_closing, HtmlParser};
use crate::tag::{Attr, Tag};
use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::Expr;

impl HtmlParser {
    /// Parse an incoming Tag::Close
    pub(crate) fn parse_close_tag(&mut self, name: Ident) {
        let idx = &mut self.current_idx;
        let parent_to_children = &mut self.parent_to_children;
        let parent_stack = &mut self.parent_stack;
        let tokens = &mut self.tokens;
        let node_order = &mut self.node_order;

        let close_span = name.span();
        let close_tag = name.to_string();

        // For example, this should have been <br /> instead of </br>
        if is_self_closing(&close_tag) {
            let error = format!(
                r#"{} is a self closing tag. Try "<{}>" or "<{} />""#,
                close_tag, close_tag, close_tag
            );
            let error = quote_spanned! {close_span=> {
                compile_error!(#error);
            }};

            tokens.push(error);
            return;
        }

        let last_open_tag = parent_stack.pop().expect("Last open tag");

        // TODO: join open and close span. Need to figure out how to enable that.
        //                let open_span = last_open_tag.1.span();

        let last_open_tag = last_open_tag.1.to_string();

        // if div != strong
        if last_open_tag != close_tag {
            let error = format!(
                r#"Wrong closing tag. Try changing "{}" into "{}""#,
                close_tag, last_open_tag
            );

            let error = quote_spanned! {close_span=> {
                compile_error!(#error);
            }};
            // TODO: Abort early if we find an error. So we should be returning
            // a Result.
            tokens.push(error);
        }
    }
}
