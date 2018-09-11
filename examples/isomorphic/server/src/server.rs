use std::net::TcpListener;

use std::io;
use std::io::Read;
use std::io::Write;

use isomorphic_app::App;
use std::fs::File;
use std::net::TcpStream;
use std::prelude::v1::Vec;
use std::string::String;
use std::fs;

pub fn serve() {
    let listener = TcpListener::bind("0.0.0.0:7878").unwrap();
    println!("Listening on port 7878");

    for stream in listener.incoming() {
        println!("Incoming connection\n\n");

        let stream = stream.unwrap();

        handle_connection(stream);
    }

    fn handle_connection(mut stream: TcpStream) {
        let mut buffer = [0; 512];
        stream.read(&mut buffer).unwrap();

        let request_pieces = String::from_utf8_lossy(&buffer);
        let buffer_words = request_pieces.split("/").collect::<Vec<&str>>();

        // Not sure what this request is but it's breaking stuff... so we ignore it..
        // Worry about this later..
        if buffer.len() < 5 || buffer_words.len() < 2 {
            let response = format!("HTTP/1.1 404 NOT FOUND\r\n\r\n");
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();

            return;
        }

        let filename = buffer_words[1];

        let filename = filename.split(" ").collect::<Vec<&str>>();
        let filename = filename[0];

        let filename = &format!("./examples/isomorphic/client/{}", filename);

        let get_home = b"GET / HTTP/1.1\r\n";

        if buffer.starts_with(get_home) {
            stream.write(b"HTTP/1.1 200 OK\r\n\r\n").unwrap();
            let app = App::new(1001);
            let state = app.state.borrow();

            let html = format!("{}", include_str!("./index.html"));
            let html = html.replacen(
                "#HTML_INSERTED_HERE_BY_SERVER#",
                &app.render().to_string(),
                1,
            );
            let html = html.replacen("#INITIAL_STATE_JSON#", &state.to_json(), 1);

            stream.write(html.as_bytes()).unwrap();
        } else {
            let file_metadata = fs::metadata(filename);

            if let Ok(file_metadata) = file_metadata {
                let mut file = File::open(filename).unwrap();

                stream.write(b"HTTP/1.1 200 OK").unwrap();

                let content_length = file_metadata.len();
                let content_length = format!("\r\nContent-Length: {}", content_length);
                stream.write(content_length.as_bytes()).unwrap();

                if String::from_utf8_lossy(&buffer).contains(".module.wasm") {
                    stream.write(b"\r\nContent-Type: application/wasm").unwrap();
                }
                stream.write(b"\r\n\r\n").unwrap();
                io::copy(&mut file, &mut stream).expect("Couldn't pipe file");
            } else {
                stream
                    .write(format!("HTTP/1.1 404 Not Found\r\n\r\n").as_bytes())
                    .unwrap();
            };
        }
        stream.flush().unwrap();
    }
}
