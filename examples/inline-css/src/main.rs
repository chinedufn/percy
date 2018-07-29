#![feature(use_extern_macros)]
#![feature(proc_macro_non_items)]

extern crate inline_stylesheet_macro;
use inline_stylesheet_macro::css;
#[macro_use]
extern crate virtual_dom_rs;

static SOME_COMPONENT_CSS: &'static str = css! {"
:host {
    font-size: 30px;
    font-weight: bold
}
"};

fn main() {
    let html = render_app();

    println!("{}", html);
}

fn render_app() -> virtual_dom_rs::VirtualNode {
    let another_component_css = css! {r#"
    :host {
        display: flex;
        flex-direction: column;
    }
    "#};

    let another_component_css = &format!("{} more classes can go here", another_component_css);

    let some_component = html! {
    <h1 class=*SOME_COMPONENT_CSS,>
    </h1>
    };

    let another_component = html! {
    <div class=another_component_css,>
    </div>
    };

    html! { <div> {some_component} {another_component} </div>}
}