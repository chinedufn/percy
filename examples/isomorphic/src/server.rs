use std::net::TcpListener;

use std::io::Write;
use std::io::Read;

pub fn serve() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Listening on port 7878");

    for stream in listener.incoming() {
        println!("Incoming connection");

        let html = html! { <div> { "Hello world" } </div> };
        let html = html.to_string();

        let mut buffer = [0; 512];
        let mut stream = stream.unwrap();

        stream.read(&mut buffer).unwrap();

        let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", html);

        stream.write(response.as_bytes()).unwrap();
        stream.flush().unwrap();
    }
}