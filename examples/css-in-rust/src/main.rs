#![feature(use_extern_macros)]
#![feature(proc_macro_non_items)]

extern crate css_rs_macro;
use css_rs_macro::css;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::net::TcpListener;

#[macro_use]
extern crate virtual_dom_rs;

static SOME_COMPONENT_CSS: &'static str = css! {"
:host {
    font-size: 30px;
    font-weight: bold;
}
"};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:22217").unwrap();

    println!("Visit http://127.0.0.1:22217 in your browser");

    for stream in listener.incoming() {
        let mut stream = stream.unwrap();

        let mut buffer = [0; 512];
        stream.read(&mut buffer).unwrap();

        let html_req = b"GET / HTTP/1.1\r\n";
        let css_req = b"GET /app.css HTTP/1.1\r\n";

        let response = "HTTP/1.1 200 OK\r\n\r\n".to_string();

        if buffer.starts_with(html_req) {
            let html = render_app();

            stream.write(&(response + &html.to_string()).into_bytes());
        } else if buffer.starts_with(css_req) {
            // Serve our CSS
            let mut css_file = File::open("examples/inline-css/app.css").unwrap();
            let mut css = String::new();

            css_file.read_to_string(&mut css).unwrap();

            stream.write(&(response + &css).into_bytes()).unwrap();
        } else {
            stream.write(&(response + "ok").into_bytes()).unwrap();
        }

        stream.flush().unwrap();
    }
}

fn render_app() -> virtual_dom_rs::VirtualNode {
    let another_component_css = css! {r#"
    :host {
        display: flex;
        flex-direction: column;
    }

    :host > h3 {
        color: blue;
    }

    .red {
        color: red;
    }
    "#};

    let another_component_css = &format!("{} more classes can go here", another_component_css);

    let some_component = html! {
    <h1 class=*SOME_COMPONENT_CSS,>
        { "And there we have it" }
    </h1>
    };

    let another_component = html! {
    <div class=another_component_css,>
        <h3> { "we have some" } </h3>
        <span class="red",> {"CSS!"} </span>
    </div>
    };

    html! { <div>
      {some_component}
      {another_component}
      <link rel="stylesheet", type="text/css", href="/app.css",></link>
    </div>}
}
