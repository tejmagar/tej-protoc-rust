use std::future::Future;
use std::sync::Arc;
use std::vec;

use tokio::io::{AsyncReadExt, AsyncWriteExt, ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

pub type Stream = Box<dyn AbstractStream + Send + Sync + Unpin>;

pub type StreamResult<'a, T> = Box<dyn Future<Output = T>  + Send + Sync + Unpin + 'a>;

pub trait AbstractStream {
    fn buffer_size(&self) -> StreamResult<usize>;
    fn read_chunk(&self) -> StreamResult<std::io::Result<Vec<u8>>>;
    fn write_chunk(&self) -> StreamResult<std::io::Result<()>>;
    fn shutdown(&self) -> StreamResult<std::io::Result<()>>;
}

pub struct TcpStreamWrapper {
    std_tcp_stream: Arc<Mutex<std::net::TcpStream>>,
    reader: Arc<Mutex<ReadHalf<TcpStream>>>,
    writer: Arc<Mutex<WriteHalf<TcpStream>>>,
    buffer_size: usize
}

impl TcpStreamWrapper {
    pub fn new(tcp_stream: TcpStream, buffer_size: usize) -> std::io::Result<Self> {
        let std_tcp_stream = tcp_stream.into_std()?;
        let cloned_std_tcp_stream = std_tcp_stream.try_clone()?;
        let async_tcp_stream = TcpStream::from_std(cloned_std_tcp_stream)?;


        let (reader, writer) = tokio::io::split(async_tcp_stream);
        Ok(Self {
            std_tcp_stream: Arc::new(Mutex::new(std_tcp_stream)),
            reader: Arc::new(Mutex::new(reader)),
            writer: Arc::new(Mutex::new(writer)),
            buffer_size
        })
    } 
}

impl AbstractStream for TcpStreamWrapper {
    fn buffer_size(&self) -> StreamResult<usize> {
        Box::new(Box::pin(async move {
            self.buffer_size
        }))
    }

    fn read_chunk(&self) -> StreamResult<std::io::Result<Vec<u8>>> {
        Box::new(Box::pin(async move {
            let mut buffer = vec![0u8; self.buffer_size];
            let mut reader = self.reader.lock().await;
            let read_size = reader.read(&mut buffer).await?;
            let chunk = &buffer[0..read_size];
            Ok(chunk.to_vec())
        }))
    }

    fn write_chunk(&self) -> StreamResult<std::io::Result<()>> {
        Box::new(Box::pin(async move {
            let mut buffer = vec![0u8; self.buffer_size];
            let mut writer = self.writer.lock().await;
            writer.write(&mut buffer).await?;
            Ok(())
        }))
    }

    fn shutdown(&self) -> StreamResult<std::io::Result<()>> {
        Box::new(Box::pin(async move {
            let std_tcp_stream = self.std_tcp_stream.lock().await;
            std_tcp_stream.shutdown(std::net::Shutdown::Both)?;
            Ok(())
        }))
    }
}
