use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt, Result};
use bytes::BytesMut;
use std::str;

pub struct ClushServer {
    listener: TcpListener,
}

impl ClushServer {
    /// create a clush server with the given TCP listener
    pub fn new(listener: TcpListener) -> ClushServer {
        ClushServer{
            listener,
        }
    }

    /// init a clush server with the default configuration
    pub async fn init() -> Result<ClushServer> {
        let listener = TcpListener::bind("0.0.0.0:9527").await?;
        Ok(ClushServer::new(listener))
    }

    /// init a clush server with the given address
    ///
    /// # Examples
    ///
    /// ```
    /// let server = ClushServer::init_with_addr(env::args().nth(1).unwrap()).await?;
    /// server.start().await
    /// ```
    pub async fn init_with_addr(addr: String) -> Result<ClushServer> {
        let listener = TcpListener::bind(addr).await?;
        Ok(ClushServer::new(listener))
    }

    /// start the event loop
    pub async fn start(&self) -> Result<()> {
        loop {
            let (mut stream, _addr) = self.listener.accept().await?;
            tokio::spawn(async move {
                process(&mut stream).await;
            });
        }
    }
}

/// process the stream
async fn process(stream: &mut TcpStream) {
    let mut buf = BytesMut::with_capacity(4096);
    loop {
        let n = stream.read_buf(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }
        parse(&buf[..n]).await
    }
}

// TODO: parse to ClushFrame
/// parse the content and convert it to ClushFrame
async fn parse(content: &[u8]) {
    println!("{}", str::from_utf8(content).unwrap());
}

#[cfg(test)]
mod tests {
    use super::*;
}