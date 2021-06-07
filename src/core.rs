use crate::core::config::*;
use crate::util::*;
use bytes::{Bytes, BytesMut};
use rbatis::rbatis::Rbatis;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, Result};
use tokio::net::{TcpListener, TcpStream};

/// buffer size
static BUF_SIZE: usize = 4096;

// TODO: add tokio_rustls TLS acceptor
// TODO: add integrity test for ClushServer
/// a clush server
///
/// # Example
///
/// ```
/// let server = ClushServer::init().await?;
/// server.start().await
/// ```
pub struct ClushServer {
    listener: TcpListener,
    db: Arc<Rbatis>,
}

impl ClushServer {
    /// create a clush server with the given TCP listener
    pub fn new(listener: TcpListener, db: Rbatis) -> ClushServer {
        let db = Arc::new(db);

        ClushServer { listener, db }
    }

    /// init a clush server with the given configuration
    ///
    /// # Example
    ///
    /// ```
    /// let config = ClushConfig::from_json("config/clush.json").await;
    /// let server = ClushServer::init_with_config(config).await?;
    /// server.start().await
    /// ```
    pub async fn init_with_config(config: ClushConfig) -> Result<ClushServer> {
        let listener = TcpListener::bind(&config.server_config.url).await?;
        let _ = fast_log::init_log(
            &config.rbatis_config.log_path,
            config.rbatis_config.log_limit,
            config.rbatis_config.log_level(),
            None,
            config.rbatis_config.debug_mode,
        );
        let db = Rbatis::new();
        db.link(&config.rbatis_config.db_url).await.unwrap();

        Ok(ClushServer::new(listener, db))
    }

    /// start the event loop
    pub async fn start(&self) -> Result<()> {
        loop {
            let (stream, _addr) = self.listener.accept().await?;
            let db = self.db.clone();

            tokio::spawn(async move {
                let mut task = Task::new(stream, db);
                task.process().await
            });
        }
    }
}

/// a frame used to communicate with clush client and server
pub struct ClushFrame {
    pub msg_type: MessageType,
    pub from_id: u64,
    pub to_id: u64,
    pub size: u64,
    pub content: BytesMut,
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

    /// set the message type of ClushFrame
    pub fn set_msg_type(&mut self, msg_type: MessageType) -> &mut ClushFrame {
        self.msg_type = msg_type;

        self
    }

    /// append the given content to ClushFrame's content
    pub fn append(&mut self, content: &[u8]) -> &mut ClushFrame {
        self.content.extend_from_slice(&content);

        self
    }

    pub fn update_size(&mut self) {
        self.size = self.content.len() as u64;
    }

    /// convert ClushFrame to Bytes
    pub fn to_bytes(&self) -> Bytes {
        let mut bytes_mut = BytesMut::with_capacity(0);
        match self.msg_type {
            MessageType::UserMessage => bytes_mut.extend_from_slice(&u32_to_bytes(&1)[..]),
            MessageType::GroupMessage => bytes_mut.extend_from_slice(&u32_to_bytes(&2)[..]),
            MessageType::UserFileMessage => bytes_mut.extend_from_slice(&u32_to_bytes(&3)[..]),
            MessageType::GroupFileMessage => bytes_mut.extend_from_slice(&u32_to_bytes(&4)[..]),
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
    db: Arc<Rbatis>,
}

impl Task {
    /// create a task to process the given stream
    fn new(stream: TcpStream, db: Arc<Rbatis>) -> Task {
        Task { stream, db }
    }

    /// process the stream
    async fn process(&mut self) -> Result<()> {
        while let Some(frame) = self.read_frame().await? {
            self.process_frame(&frame).await?;
        }
        // TODO: offline after the task is done

        Ok(())
    }

    /// read a frame from the stream
    async fn read_frame(&mut self) -> Result<Option<ClushFrame>> {
        let mut buf = BytesMut::with_capacity(BUF_SIZE);

        let n = self.stream.read_buf(&mut buf).await?;

        // if is keep-alive
        if n == 0 {
            return Ok(None);
        }

        // length of msg_type + from_id + to_id + size
        if n < 28 {
            panic!("incomplete package!")
        }

        // convert numbers to bytes
        let from_id = u64_from_bytes(&buf[4..12]).unwrap();
        let to_id = u64_from_bytes(&buf[12..20]).unwrap();
        let size = u64_from_bytes(&buf[20..28]).unwrap();
        // construct a frame of undefined type
        let mut frame = ClushFrame::new(
            MessageType::Undefined,
            from_id,
            to_id,
            size,
            BytesMut::from(&buf[28..n]),
        );
        // get the message type id, set frame message type according to it
        match u32_from_bytes(&buf[0..4]).unwrap() {
            1 => frame.set_msg_type(MessageType::UserMessage),
            2 => frame.set_msg_type(MessageType::GroupMessage),
            3 => frame.set_msg_type(MessageType::UserFileMessage),
            4 => frame.set_msg_type(MessageType::GroupFileMessage),
            _ => return Ok(None),
        };

        // read the last of stream and add to content
        loop {
            let n = self.stream.read_buf(&mut buf).await?;
            if n == 0 {
                // check data integrity
                if frame.content.len() as u64 != frame.size {
                    panic!("data mismatch!");
                }
                break;
            }
            frame.append(&buf[..n]);
        }

        Ok(Some(frame))
    }

    /// write a frame to the stream
    async fn write_frame(&mut self, frame: ClushFrame) -> Result<()> {
        self.stream.write(&frame.to_bytes()[..]).await?;

        Ok(())
    }

    // TODO: implement process
    /// process the frame according to the frame type
    async fn process_frame(&self, frame: &ClushFrame) -> Result<()> {
        match frame.msg_type {
            _ => panic!("unimplemented!"),
        }
    }
}

pub mod config {
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;
    use tokio::fs::File;
    use tokio::io::AsyncReadExt;

    /// all configuration
    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ClushConfig {
        pub server_config: ServerConfig,
        pub rbatis_config: RbatisConfig,
    }

    impl ClushConfig {
        pub async fn from_json(path: &str) -> ClushConfig {
            let mut file = File::open(path).await.unwrap();
            let mut content = vec![];
            file.read_to_end(&mut content).await.unwrap();
            let obj: ClushConfig = serde_json::from_slice(&content).unwrap();

            obj
        }
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct ServerConfig {
        pub url: String,
        pub enable_tls: bool,
        pub key_path: String,
        pub cert_path: String,
    }

    #[derive(Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct RbatisConfig {
        pub db_url: String,
        pub log_path: String,
        pub log_level: String,
        pub log_limit: usize,
        pub debug_mode: bool,
    }

    impl RbatisConfig {
        pub fn log_level(&self) -> log::Level {
            log::Level::from_str(&self.log_level).unwrap()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clush_frame_test() {
        let frame = ClushFrame::new(
            MessageType::UserMessage,
            0,
            0,
            5,
            BytesMut::from(&"hello"[..]),
        );
        assert_eq!(0, frame.from_id);
        assert_eq!(0, frame.to_id);
        assert_eq!(5, frame.size);
        assert_eq!(BytesMut::from(&"hello"[..]), frame.content);
        assert_eq!(
            Bytes::from(
                &[
                    0u8, 0u8, 0u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
                    0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 5u8, b'h', b'e', b'l',
                    b'l', b'o'
                ][..]
            ),
            frame.to_bytes()
        );
    }

    #[test]
    fn update_size_test() {
        let mut frame = ClushFrame::new(
            MessageType::UserMessage,
            0,
            0,
            0,
            BytesMut::from(&"hello"[..]),
        );
        frame.update_size();
        assert_eq!(5, frame.size);
    }
}
