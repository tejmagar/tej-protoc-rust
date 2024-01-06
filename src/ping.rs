use std::io::Write;
use std::net::TcpStream;
use std::sync::{Arc, Mutex, RwLock, RwLockWriteGuard};
use std::thread::{self, JoinHandle, sleep};
use std::time::Duration;
use crate::protoc::encoder::build_raw_bytes;
use crate::protoc::{File, StatusCode};

pub fn ping(mut tcp_stream: Arc<RwLock<TcpStream>>, sleep_duration: Duration) -> JoinHandle<()> {
    let files: Vec<&File> = Vec::new();
    let message: Vec<u8> = Vec::new();
    let ping_bytes = build_raw_bytes(1, StatusCode::Ping as u8, 1, &files, &message);

    return thread::spawn(move || {
        loop {
            sleep(sleep_duration);
            tcp_stream.write().unwrap().write_all(&ping_bytes).unwrap();
        }
    });
}
