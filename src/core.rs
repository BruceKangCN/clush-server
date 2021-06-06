use crate::util::*;
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

    /// convert ClushFrame to Bytes
    pub fn to_bytes(&self) -> Bytes {
        let mut bytes_mut = BytesMut::with_capacity(0);
        match self.msg_type {
            _ => bytes_mut.extend_from_slice(&u32_to_bytes(&0)[..]),
        }
        bytes_mut.extend_from_slice(&u64_to_bytes(&self.from_id)[..]);
        bytes_mut.extend_from_slice(&u64_to_bytes(&self.to_id)[..]);
        bytes_mut.extend_from_slice(&u64_to_bytes(&self.size)[..]);
        bytes_mut.extend_from_slice(&self.content[..]);

        bytes_mut.freeze()
    }
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
        while let Some(frame) = self.read_frame().await? {
            self.process_frame(&frame).await.unwrap();
        }

        Ok(())
    }

    /// read a frame from the stream
    pub async fn read_frame(&mut self) -> Result<Option<ClushFrame>> {
        let mut buf = BytesMut::with_capacity(BUF_SIZE);

        let n = self.stream.read_buf(&mut buf).await?;

        // if is keep-alive
        if n == 0 {
            return Ok(Some(ClushFrame::new(
                MessageType::None,
                0,
                0,
                0,
                BytesMut::from(""),
            )));
        }

        // length of msg_type + from_id + to_id + size
        if n < 28 {
            panic!("incomplete package!")
        }

        // convert numbers to bytes
        let from_id = u64_from_bytes(&buf[4..12]).unwrap();
        let to_id = u64_from_bytes(&buf[12..20]).unwrap();
        let size = u64_from_bytes(&buf[20..28]).unwrap();

        // get the message type id, construct a frame according to it
        let mut frame = match u32_from_bytes(&buf[0..4]).unwrap() {
            0 => return Ok(None),
            _ => ClushFrame::new(
                MessageType::None,
                from_id,
                to_id,
                size,
                BytesMut::from(&buf[28..n]),
            ), // TODO: convert u32 to enum
        };

        // read the last of stream and add to content
        loop {
            let n = self.stream.read_buf(&mut buf).await?;
            if n == 0 {
                // check data integrity
                if frame.content.len() as u64 != frame.size() {
                    panic!("data mismatch!");
                }
                break;
            }
            frame.append(&buf[..n]);
        }

        Ok(Some(frame))
    }

    /// write a frame to the stream
    pub async fn write_frame(&mut self, frame: ClushFrame) -> Result<()> {
        self.stream.write(&frame.to_bytes()[..]).await.unwrap();

        Ok(())
    }

    // TODO: implement process
    /// process the frame according to the frame type
    async fn process_frame(&self, frame: &ClushFrame) -> Result<()> {
        // println!("{}", str::from_utf8(content).unwrap());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
}
