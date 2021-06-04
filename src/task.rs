use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt, Result};
use bytes::BytesMut;
use std::str;

pub struct Task {
    stream: TcpStream,
}

impl Task {
    /// create a task to process the given stream
    pub fn new(stream: TcpStream) -> Task {
        Task {
            stream,
        }
    }

    /// process the stream
    pub async fn process(&mut self) -> Result<()> {
        let mut buf = BytesMut::with_capacity(4096);
        loop {
            let n = self.stream.read_buf(&mut buf).await?;
            if n == 0 {
                break;
            }
            self.parse(&buf[..n]).await
        }

        Ok(())
    }

    // TODO: parse to ClushFrame
    /// parse the content and convert it to ClushFrame
    pub async fn parse(&self, content: &[u8]) {
        println!("{}", str::from_utf8(content).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
