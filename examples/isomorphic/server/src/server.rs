use std::net::TcpListener;

use std::io::Read;
use std::io::Write;

use isomorphic_app::App;
use std::fs::File;
use std::io::prelude::*;
use std::net::TcpStream;
use std::prelude::v1::Vec;
use std::string::String;

pub fn serve() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Listening on port 7878");

    for stream in listener.incoming() {
        println!("Incoming connection");

        let stream = stream.unwrap();

        handle_connection(stream);
    }

    fn handle_connection(mut stream: TcpStream) {
        let mut buffer = [0; 512];
        stream.read(&mut buffer).unwrap();

        //        println!("{}", String::from_utf8_lossy(&buffer));

        let get_home = b"GET / HTTP/1.1\r\n";
        let wasm = b"GET /078fc2bb27ecd130333c.module.wasm HTTP/1.1\r\n";
        let bundle0 = b"GET /0.bundle.js HTTP/1.1\r\n";
        let bundle1 = b"GET /1.bundle.js HTTP/1.1\r\n";

        if buffer.starts_with(get_home) {
            let app = App::new();

            let html = format!("{}", include_str!("./index.html"));
            let html = html.replace("#HTML_INSERTED_HERE_BY_SERVER#", &app.render().to_string());

            let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", html);

            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        } else if buffer.starts_with(wasm) {
            let wasm = include_bytes!("../../client/078fc2bb27ecd130333c.module.wasm");

            let application_wasm = "\r\nContent-Type: application/wasm";
            stream
                .write(format!("HTTP/1.1 200 OK{}\r\n\r\n", application_wasm).as_bytes())
                .unwrap();
            stream.write(wasm).unwrap();
            stream.flush().unwrap();
        } else {
            let request_pieces = String::from_utf8_lossy(&buffer);
            let buffer_words = request_pieces.split("/").collect::<Vec<&str>>();
            let filename = buffer_words[1];

            let filename = filename.split(" ").collect::<Vec<&str>>();
            let filename = filename[0];

            println!("{}", ::std::env::current_dir().unwrap().display());
            let filename = &format!("./examples/isomorphic/client/{}", filename);
            println!("FILENAME: {}", filename);

            let mut file = File::open(filename).expect("File not found");

            let mut contents = String::new();
            file.read_to_string(&mut contents).unwrap();

            let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", contents);
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}
