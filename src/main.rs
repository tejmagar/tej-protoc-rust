use std::io::Write;
use std::net::{Shutdown, TcpListener, TcpStream};
use std::thread;
use tej_protoc::protoc::decoder::{DecodedResponse};
use tej_protoc::protoc::encoder::{build_raw_bytes};
use tej_protoc::protoc::{decode_tcp_stream, File};


fn handle_decoded_response(tcp_stream: &mut TcpStream, decoded_response: DecodedResponse) {
    println!("{:?}", decoded_response);
}

fn test_client() {
    let mut tcp_stream = TcpStream::connect("127.0.0.1:1234").unwrap();
    println!("Connected to 127.0.0.1:1234");

    let a: Vec<&File> = Vec::new();
    let bytes = build_raw_bytes(3, 1, &a, &"Test 123".as_bytes().to_vec());
    tcp_stream.write_all(&bytes).unwrap();

    loop {
        let decoded_response = decode_tcp_stream(&mut tcp_stream);

        match decoded_response {
            Ok(response) => {
                handle_decoded_response(&mut tcp_stream, response);
            }

            Err(error) => {
                eprintln!("{}", error);

                match (tcp_stream.shutdown(Shutdown::Both)) {
                    Ok(_) => {}
                    Err(error) => {
                        eprintln!("Failed to shutdown tcp stream. {:?}", error);
                    }
                }
            }
        }
    }
}

fn test_server() {
    println!("Starting server in 127.0.0.1:1234");
    let server = TcpListener::bind("127.0.0.1:1234").unwrap();

    for tcp_stream in server.incoming() {
        thread::spawn(move || {
            print!("Connected");
            let mut tcp_stream = tcp_stream.unwrap();
            let a: Vec<&File> = Vec::new();
            let bytes = build_raw_bytes(1, 1,  &a, &"Test 123".as_bytes().to_vec());
            tcp_stream.write_all(&bytes).unwrap();

            loop {
                let result = decode_tcp_stream(&mut tcp_stream);
                println!("{:?}", result);
            }
        });
    }
}

fn main() {
    test_client();
    // test_server();
}
