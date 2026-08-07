use crate::parser::{HtmlParser, NodesToPush};
use quote::quote;
use syn::{Expr, ExprIf, Stmt};

impl HtmlParser {
    /// Parse an incoming syn::Stmt node inside a block
    pub(crate) fn parse_statement(&mut self, stmt: &Stmt, real_dom_ty: &syn::Type) {
        // Here we handle a block being a descendant within some html! call.
        //
        // The descendant should implement Into<IterableNodes>
        //
        // html { <div> { some_node } </div> }
        match stmt {
            Stmt::Expr(expr, _) => {
                self.parse_expr(stmt, expr, real_dom_ty);
            }
            _ => {
                self.push_iterable_nodes(NodesToPush::Stmt(stmt), real_dom_ty);
            }
        };
    }

    /// Parse an incoming syn::Expr node inside a block
    pub(crate) fn parse_expr(&mut self, stmt: &Stmt, expr: &Expr, real_dom_ty: &syn::Type) {
        match expr {
            Expr::If(expr_if) => {
                self.expand_if(stmt, expr_if, real_dom_ty);
            }
            _ => {
                self.push_iterable_nodes(NodesToPush::Stmt(stmt), real_dom_ty);
            }
        }
    }

    /// Expand an incoming Expr::If block
    /// This enables us to use JSX-style conditions inside of blocks such as
    /// the following example.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// html! {
    ///     <div>
    ///         {if condition_is_true {
    ///                html! { <span>Hello World</span> }
    ///         }}
    ///     </div>
    /// }
    /// ```
    ///
    /// Traditionally this would be possible as an if statement in rust is an
    /// expression, so the then, and the else block have to return matching types.
    /// Here we identify whether the block is missing the else and fill it in with
    /// a blank VirtualNode::new_text("")
    pub(crate) fn expand_if(&mut self, stmt: &Stmt, expr_if: &ExprIf, real_dom_ty: &syn::Type) {
        // Has else branch, we can parse the expression as normal.
        if let Some(_else_branch) = &expr_if.else_branch {
            self.push_iterable_nodes(NodesToPush::Stmt(stmt), real_dom_ty);
        } else {
            let condition = &expr_if.cond;
            let block = &expr_if.then_branch;
            let tokens = quote! {
                if #condition {
                    #block.into()
                } else {
                    VirtualNode::new_text("")
                }
            };

            self.push_iterable_nodes(NodesToPush::TokenStream(stmt, tokens), real_dom_ty);
        }
    }
}
