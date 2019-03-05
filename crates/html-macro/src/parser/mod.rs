use crate::Tag;
use proc_macro2::LineColumn;
use quote::{quote, quote_spanned};
use std::collections::HashMap;
use syn::export::Span;
use syn::Ident;

mod braced;
mod close_tag;
mod open_tag;
mod text;

/// Iterate over Tag's that we've parsed and build a tree of VirtualNode's
pub struct HtmlParser {
    /// As we parse our macro tokens we'll generate new tokens to return back into the compiler
    /// when we're done.
    tokens: Vec<proc_macro2::TokenStream>,
    /// Everytime we encounter a new node we'll use the current_idx to name it.
    /// Then we'll increment the current_idx by one.
    /// This gives every node that we encounter a unique name that we can use to find
    /// it later when we want to push child nodes into parent nodes
    current_idx: usize,
    /// The order that we encountered nodes while parsing.
    node_order: Vec<usize>,
    /// Each time we encounter a new node that could possible be a parent node
    /// we push it's node index onto the stack.
    ///
    /// Text nodes cannot be parent nodes.
    parent_stack: Vec<(usize, Ident)>,
    /// Key -> index of the parent node within the HTML tree
    /// Value -> vector of child node indices
    parent_to_children: HashMap<usize, Vec<usize>>,
    recent_span_locations: RecentSpanLocations,
}

/// TODO: I've hit a good stopping point... but we can clean these methods up / split them up
/// a bit...
impl HtmlParser {
    /// Create a new HtmlParser
    pub fn new() -> HtmlParser {
        let mut parent_to_children: HashMap<usize, Vec<usize>> = HashMap::new();
        parent_to_children.insert(0, vec![]);

        HtmlParser {
            tokens: vec![],
            current_idx: 0,
            node_order: vec![],
            parent_stack: vec![],
            parent_to_children,
            recent_span_locations: RecentSpanLocations::default(),
        }
    }

    /// Generate the tokens for the incoming Tag and update our parser's heuristics that keep
    /// track of information about what we've parsed.
    pub fn push_tag(&mut self, tag: Tag) {
        match tag {
            Tag::Open { name, attrs } => {
                self.parse_open_tag(name, attrs);
            }
            Tag::Close { name } => {
                self.parse_close_tag(name);
            }
            Tag::Text { text } => {
                self.parse_text(text);
            }
            Tag::Braced { block } => self.parse_braced(block),
        };
    }

    ///  1. Pop a node off the stack
    ///  2. Look up all of it's children in parent_to_children
    ///  3. Append the children to this node
    ///  4. Move on to the next node (as in, go back to step 1)
    pub fn finish(&mut self) -> proc_macro2::TokenStream {
        let node_order = &mut self.node_order;
        let parent_to_children = &mut self.parent_to_children;
        let tokens = &mut self.tokens;

        if node_order.len() > 1 {
            for _ in 0..(node_order.len()) {
                let parent_idx = node_order.pop().unwrap();

                // TODO: Figure out how to really use spans
                let parent_name =
                    Ident::new(format!("node_{}", parent_idx).as_str(), Span::call_site());

                let parent_to_children_indices = match parent_to_children.get(&parent_idx) {
                    Some(children) => children,
                    None => continue,
                };

                if parent_to_children_indices.len() > 0 {
                    for child_idx in parent_to_children_indices.iter() {
                        let children =
                            Ident::new(format!("node_{}", child_idx).as_str(), Span::call_site());

                        let unreachable = quote_spanned!(Span::call_site() => {
                            unreachable!("Non-elements cannot have children");
                        });
                        let push_children = quote! {
                            if let Some(ref mut element_node) =  #parent_name.as_velement_mut() {
                                element_node.children.extend(#children.into_iter());
                            } else {
                                #unreachable;
                            }
                        };
                        tokens.push(push_children);
                    }
                }
            }
        }

        // Create a virtual node tree
        let node = quote! {
            {
                #(#tokens)*
                // Root node is always named node_0
                node_0
            }
        };
        node
    }

    /// Set the location of the most recent start tag's ending LineColumn
    fn set_most_recent_start_tag_end (&mut self, span: Span) {
    }

    /// Set the location of the most recent start tag's ending LineColumn
    fn set_most_recent_block_start (&mut self, span: Span) {
    }
}

/// Keep track of the locations of different kinds of tokens that we encounter.
///
/// This helps us determine whether or not to insert space before or after text tokens
/// in cases such as:
///
/// ```ignore
/// html! { <div> { Hello World } </div>
/// html! { <div>{Hello World}</div>
/// ```
#[derive(Default)]
struct RecentSpanLocations {
    most_recent_start_tag_end: Option<LineColumn>,
    most_recent_block_start: Option<LineColumn>,
}

// TODO: Cache this as a HashSet inside of our parser
fn is_self_closing(tag: &str) -> bool {
    let whitelist = [
        "area", "base", "br", "col", "hr", "img", "input", "link", "meta", "param", "command",
        "keygen", "source",
    ];

    for whitelisted in whitelist.iter() {
        if &tag == whitelisted {
            return true;
        }
    }

    return false;
}
