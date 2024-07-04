use std::io::Write;
use std::sync::Arc;
use std::thread;
use tej_protoc::protoc::decoder::{decode_tcp_stream, DecodedResponse};
use tej_protoc::protoc::encoder::{build_raw_bytes};
use tej_protoc::protoc::{File};
use tej_protoc::stream::{AbstractStream, Stream, TcpStreamWrapper};
use tokio::net::{TcpListener, TcpStream};


fn handle_decoded_response(stream: Arc<Stream>, decoded_response: DecodedResponse) {
    println!("{:?}", String::from_utf8_lossy(&decoded_response.message));
}

async fn test_client() {
    let tcp_stream = TcpStream::connect("127.0.0.1:1234").await.unwrap();
    let stream: Arc<Stream> = Arc::new(Box::new(TcpStreamWrapper::new(tcp_stream, 1024).unwrap()));
    println!("Connected to 127.0.0.1:1234");

    let a: Vec<&File> = Vec::new();
    let bytes = build_raw_bytes(3, 1, &a, &"Test 123".as_bytes().to_vec());
    let _ = stream.write_chunk(&bytes).await;

    loop {
        // let v = stream.read_exact(1).await.unwrap().;

        let decoded_response = decode_tcp_stream(stream.clone()).await;

        match decoded_response {
            Ok(response) => {
                handle_decoded_response(stream.clone(), response);
            }

            Err(error) => {
                eprintln!("{}", error);

                match stream.shutdown().await {
                    Ok(_) => {}
                    Err(error) => {
                        eprintln!("Failed to shutdown tcp stream. {:?}", error);
                    }
                }
                break;
            }
        }
    }
}

async fn test_server() {
    println!("Starting server in 127.0.0.1:1234");
    let server = TcpListener::bind("127.0.0.1:1234").await.unwrap();

    loop {
        let (tcp_stream, _) = server.accept().await.unwrap();

        tokio::spawn(async move {
            print!("Connected");
            let stream: Arc<Stream> = Arc::new(Box::new(TcpStreamWrapper::new(tcp_stream, 1024).unwrap()));
            let a: Vec<&File> = Vec::new();
            let bytes = build_raw_bytes(1, 1,  &a, &"Test 123".as_bytes().to_vec());
            stream.write_chunk(&bytes).await.unwrap();

            loop {
                let result = decode_tcp_stream(stream.clone()).await;
                println!("{:?}", result);
            }
        });
    }
}

#[tokio::main]
async fn main() {
    // test_client().await;
    test_server().await;
}
