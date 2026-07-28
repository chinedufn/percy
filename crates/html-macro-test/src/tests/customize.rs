use html_macro::define_html_macro;
use virtual_node::VirtualNode;

define_html_macro! {
    /// This macro is used to confirm that `define_html_macro!` works.
    custom!
    real_dom = (),
    calls = html_macro::html_with_config,
}

/// Verify that we can call a macro created by [`define_html_macro`].
#[test]
fn custom_macro() {
    let node = custom! { <div></div> };
    assert_eq!(node, VirtualNode::new_element("div"));
}
