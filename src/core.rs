use crate::util::{u32_from_bytes, u64_from_bytes, MessageType};
use bytes::{Bytes, BytesMut};
use tokio::io::{AsyncReadExt, AsyncWriteExt, Result};
use tokio::net::{TcpListener, TcpStream};

static BUF_SIZE: usize = 4096;

// TODO: add tokio_rustls TLS acceptor
/// a clush server
///
/// # Examples
///
/// ```
/// let server = ClushServer::init().await?;
/// server.start().await
/// ```
pub struct ClushServer {
    listener: TcpListener,
}

impl ClushServer {
    /// create a clush server with the given TCP listener
    pub fn new(listener: TcpListener) -> ClushServer {
        ClushServer { listener }
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
            let (stream, _addr) = self.listener.accept().await?;
            tokio::spawn(async move {
                let mut task = Task::new(stream);
                task.process().await
            });
        }
    }
}

/// a frame used to communicate with clush client and server
pub struct ClushFrame {
    msg_type: MessageType,
    from_id: u64,
    to_id: u64,
    size: u64,
    content: BytesMut,
}

impl ClushFrame {
    pub fn new(
        msg_type: MessageType,
        from_id: u64,
        to_id: u64,
        size: u64,
        content: BytesMut,
    ) -> ClushFrame {
        ClushFrame {
            msg_type,
            from_id,
            to_id,
            size,
            content,
        }
    }

    /// append the given content to ClushFrame's content
    pub fn append(&mut self, content: &[u8]) -> &mut ClushFrame {
        self.content.extend_from_slice(&content);

        self
    }

    /// get content part size
    pub fn size(&self) -> u64 {
        self.size
    }

    /// get content part of the ClushFrame
    pub fn content(&self) -> Bytes {
        Bytes::from(self.content.clone())
    }

    // TODO: implement read, write, process
    // pub async fn read_frame(stream: &mut TcpStream) -> ClushFrame {

    // }

    // pub async fn write_frame(stream: &mut TcpStream) -> Result<()> {

    // }

    // pub fn process(&mut self) -> Result<()> {
    //     Ok(())
    // }
}

/// a task to process the given TcpStream
struct Task {
    stream: TcpStream,
}

impl Task {
    /// create a task to process the given stream
    fn new(stream: TcpStream) -> Task {
        Task { stream }
    }

    /// process the stream
    async fn process(&mut self) -> Result<()> {
        let mut buf = BytesMut::with_capacity(BUF_SIZE);

        let n = self.stream.read_buf(&mut buf).await?;
        if n == 0 {
            return Ok(());
        }

        // length of msg_type + from_id + to_id + size
        if n < 28 {
            panic!("incomplete package!")
        }

        let from_id = u64_from_bytes(&buf[4..12]).unwrap();
        let to_id = u64_from_bytes(&buf[12..20]).unwrap();
        let size = u64_from_bytes(&buf[20..28]).unwrap();
        let mut frame = match u32_from_bytes(&buf[0..4]) {
            _ => ClushFrame::new(
                MessageType::None,
                from_id,
                to_id,
                size,
                BytesMut::from(&buf[28..n]),
            ), // TODO: convert u32 to enum
        };

        loop {
            let n = self.stream.read_buf(&mut buf).await?;
            if n == 0 {
                if frame.content.len() as u64 != frame.size() {
                    panic!("data mismatch!");
                }
                break;
            }
            frame.append(&buf[..n]);
        }
        self.parse(&frame).await;

        Ok(())
    }

    // TODO: implement get_frame
    // process(){while let frame = get_frame() {frame.process()}}

    // TODO: parse to ClushFrame
    /// parse the content and convert it to ClushFrame
    async fn parse(&self, frame: &ClushFrame) {
        // println!("{}", str::from_utf8(content).unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
