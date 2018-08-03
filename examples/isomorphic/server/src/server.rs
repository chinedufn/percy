use std::net::TcpListener;

use std::io;
use std::io::Read;
use std::io::Write;

use isomorphic_app::App;
use std::fs::File;
use std::net::TcpStream;
use std::prelude::v1::Vec;
use std::string::String;

pub fn serve() {
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    println!("Listening on port 7878");

    for stream in listener.incoming() {
        println!("Incoming connection\n\n");

        let stream = stream.unwrap();

        handle_connection(stream);
    }

    fn handle_connection(mut stream: TcpStream) {
        eprintln!("handling connection = ");
        let mut buffer = [0; 512];
        stream.read(&mut buffer).unwrap();

        let request_pieces = String::from_utf8_lossy(&buffer);
        let buffer_words = request_pieces.split("/").collect::<Vec<&str>>();

        println!("REQUEST BUFFER: {}\n\n", String::from_utf8_lossy(&buffer));

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
        println!("FILENAME: {}\n\n", filename);

        let get_home = b"GET / HTTP/1.1\r\n";

        if buffer.starts_with(get_home) {
            let app = App::new(1001);
            let state = app.state.borrow();

            let html = format!("{}", include_str!("./index.html"));
            let html = html.replace("#HTML_INSERTED_HERE_BY_SERVER#", &app.render().to_string());
            let html = html.replace("#INITIAL_STATE_JSON#", &state.to_json());

            let response = format!("HTTP/1.1 200 OK\r\n\r\n{}", html);

            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        } else if String::from_utf8_lossy(&buffer).contains(".module.wasm") {
            let mut wasm_file = File::open(filename).expect("Wasm file not found");

            let application_wasm = "\r\nContent-Type: application/wasm";
            stream
                .write(format!("HTTP/1.1 200 OK{}\r\n\r\n", application_wasm).as_bytes())
                .unwrap();
            io::copy(&mut wasm_file, &mut stream).expect("Couldn't pipe wasm file");
            stream.flush().unwrap();
        } else {
            let response = if let Ok(mut file) = File::open(filename) {
                let mut contents = String::new();
                file.read_to_string(&mut contents).unwrap();
                format!("HTTP/1.1 200 OK\r\n\r\n{}", contents)
            } else {
                format!("HTTP/1.1 404 Not Found\r\n\r\n")
            };
            stream.write(response.as_bytes()).unwrap();
            stream.flush().unwrap();
        }
    }
}
