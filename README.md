# tej-protoc

A custom tej_protocol implemented in Rust language for fast file and message transfer. 
Also, checkout [Python](https://github.com/tejmagar/tej-protoc) implementation.

## Example Usage

### Client
Simple client demo

```rust
use std::net::{Shutdown, TcpStream};
use tej_protoc::protoc::decoder::{decode_tcp_stream, DecodedResponse};


fn handle_decoded_response(tcp_stream: &mut TcpStream, decoded_response: DecodedResponse) {
    println!("{:?}", decoded_response);
}

fn test_client() {
    let mut tcp_stream = TcpStream::connect("127.0.0.1:1234").unwrap();

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
```


### Server
```rust
fn test_server() {
    println!("Starting server in 127.0.0.1:1234");
    let server = TcpListener::bind("127.0.0.1:1234").unwrap();

    for tcp_stream in server.incoming() {
        thread::spawn(move || {
            print!("Connected");
            let mut tcp_stream = tcp_stream.unwrap();
            let bytes = build_bytes_for_message(&"Test 123".as_bytes().to_vec());
            tcp_stream.write_all(&bytes).unwrap();

            loop {
                let result = decode_tcp_stream(&mut tcp_stream);
                println!("{:?}", result);
            }
        });
    }
}
```

