use crate::config::ClushConfig;
use crate::entity::*;
use crate::util::*;
use bytes::{Bytes, BytesMut};
use dashmap::DashMap;
use rbatis::crud::CRUD;
use rbatis::rbatis::Rbatis;
use std::ops::DerefMut;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, Result};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::mpsc;

/// buffer size
static BUF_SIZE: usize = 4096;

// TODO: add tokio_rustls TLS acceptor
// TODO: add integrity test for ClushServer
// TODO: add group_map to store online member of a group
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
    map: Arc<DashMap<u64, Task>>,
}

impl ClushServer {
    /// create a clush server with the given TCP listener
    pub fn new(listener: TcpListener, db: Rbatis) -> ClushServer {
        // wrap in Arc for using in multi-threading context
        let db = Arc::new(db);
        let map = Arc::new(DashMap::new());

        ClushServer { listener, db, map }
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
        // create listener
        let listener = TcpListener::bind(&config.server_config.url).await?;
        // create logger
        fast_log::init_log(
            &config.rbatis_config.log_path,
            config.rbatis_config.log_limit,
            config.rbatis_config.log_level(),
            None,
            config.rbatis_config.debug_mode,
        )
        .unwrap();
        // create database connection pool
        let db = Rbatis::new();
        db.link(&config.rbatis_config.db_url).await.unwrap();

        Ok(ClushServer::new(listener, db))
    }

    /// start the event loop
    pub async fn start(&self) -> Result<()> {
        // create a channel to handle message
        let (tx, rx) = mpsc::channel::<ClushFrame>(1024);
        let map = self.map.clone();

        // spawn a task to read message
        tokio::spawn(async move {
            let mut handler = MessageHandler::new(rx, map);
            while let Some(frame) = handler.rx.recv().await {
                match frame.msg_type {
                    MessageType::UserMessage => handler.handle_user_msg(frame).await,
                    _ => (),
                }
            }
        });

        // main event loop
        loop {
            // get stream from listener
            let (stream, _addr) = self.listener.accept().await?;
            // clone Arc of db, map to use in new task
            let db = self.db.clone();
            let map = self.map.clone();
            // clone sender to use in task
            let tx = tx.clone();

            // spawn a new task
            tokio::spawn(async move {
                // create a new task to deal with the stream
                let mut task = Task::new(stream, db, tx);

                // first login to server
                if let Some(uid) = task.process_login().await {
                    // store task to map if login success
                    map.insert(uid, task);

                    // write back a success information if login succeed
                    let frame = ClushFrame::new(
                        MessageType::LoginMessage,
                        0,
                        uid,
                        0,
                        BytesMut::from("success"),
                    );
                    map.get_mut(&uid).unwrap().write_frame(frame).await.unwrap();

                    // then start to process the rest
                    if let Some(mut task) = map.get_mut(&uid) {
                        task.process().await.unwrap();
                    }

                    // remove task when it is done
                    map.remove(&uid);
                } else {
                    // write back a failure message if login fail
                    let frame = ClushFrame::new(
                        MessageType::LoginMessage,
                        0,
                        0,
                        0,
                        BytesMut::from("failed"),
                    );
                    task.write_frame(frame).await.unwrap();
                }
            });
        }
    }
}

/// a frame used to communicate with clush client and server
#[derive(Clone, Debug)]
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
    pub fn set_msg_type(&mut self, msg_type: MessageType) -> &mut Self {
        self.msg_type = msg_type;

        self
    }

    /// append the given content to ClushFrame's content
    pub fn append(&mut self, content: &[u8]) -> &mut Self {
        self.content.extend_from_slice(&content);

        self
    }

    /// update the size of frame
    pub fn update_size(&mut self) {
        self.size = self.content.len() as u64;
    }

    /// convert ClushFrame to Bytes
    /// convenience for writing ClushFrame into byte stream
    pub fn to_bytes(&self) -> Bytes {
        let mut bytes_mut = BytesMut::with_capacity(0);

        // convert MessageType to [u8; 4]
        match self.msg_type {
            MessageType::UserMessage => bytes_mut.extend_from_slice(&u32_to_bytes(1)[..]),
            MessageType::GroupMessage => bytes_mut.extend_from_slice(&u32_to_bytes(2)[..]),
            MessageType::UserFileMessage => bytes_mut.extend_from_slice(&u32_to_bytes(3)[..]),
            MessageType::GroupFileMessage => bytes_mut.extend_from_slice(&u32_to_bytes(4)[..]),
            _ => bytes_mut.extend_from_slice(&u32_to_bytes(0)[..]),
        }

        bytes_mut.extend_from_slice(&u64_to_bytes(self.from_id)[..]);
        bytes_mut.extend_from_slice(&u64_to_bytes(self.to_id)[..]);
        bytes_mut.extend_from_slice(&u64_to_bytes(self.size)[..]);
        bytes_mut.extend_from_slice(&self.content[..]);

        bytes_mut.freeze()
    }
}

/// a task to process the given TcpStream
struct Task {
    stream: TcpStream,
    db: Arc<Rbatis>,
    tx: mpsc::Sender<ClushFrame>,
}

impl Task {
    /// create a task to process the given stream
    fn new(stream: TcpStream, db: Arc<Rbatis>, tx: mpsc::Sender<ClushFrame>) -> Task {
        Task { stream, db, tx }
    }

    /// process the stream
    async fn process(&mut self) -> Result<()> {
        while let Some(frame) = self.read_frame().await? {
            // TODO: handle frame with service
            self.process_frame(frame).await?;
        }

        Ok(())
    }

    /// read a frame from the stream
    async fn read_frame(&mut self) -> Result<Option<ClushFrame>> {
        let mut buf = BytesMut::with_capacity(BUF_SIZE);

        // get the amount of bytes read
        let n = self.stream.read_buf(&mut buf).await?;

        // if is keep-alive
        if n == 0 {
            return Ok(Some(ClushFrame::new(
                MessageType::Undefined,
                0,
                0,
                0,
                BytesMut::with_capacity(0),
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
            0 => frame.set_msg_type(MessageType::LoginMessage),
            1 => frame.set_msg_type(MessageType::UserMessage),
            2 => frame.set_msg_type(MessageType::GroupMessage),
            3 => frame.set_msg_type(MessageType::UserFileMessage),
            4 => frame.set_msg_type(MessageType::GroupFileMessage),
            _ => return Ok(None),
        };

        // if all data is received, return, otherwise start loop
        if frame.size == frame.content.len() as u64 {
            return Ok(Some(frame));
        }

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

    /// process the login message,
    /// return Some(uid) if login success,
    /// or None if failed
    async fn process_login(&mut self) -> Option<u64> {
        // receive the first frame
        if let Some(first_frame) = self.read_frame().await.unwrap() {
            // handle login message, panic if others
            match first_frame.msg_type {
                MessageType::LoginMessage => {
                    // get uid, password from frame
                    let uid = first_frame.from_id;
                    let password = first_frame.content.freeze();

                    // get user info from database
                    let user = self.db.fetch_by_id::<User>("", &uid).await.unwrap();
                    // check password
                    if user.password.unwrap() == password {
                        // TODO: fetch messages

                        Some(uid)
                    } else {
                        // if mismatch, send back an error frame
                        let mut err_frame = ClushFrame::new(
                            MessageType::UserMessage,
                            0,
                            uid,
                            0,
                            BytesMut::from("invalid password"),
                        );
                        err_frame.update_size();
                        self.write_frame(err_frame).await.unwrap();

                        None
                    }
                }
                _ => panic!("invalid login message"),
            }
        } else {
            // if failed to receive first frame, return None
            None
        }
    }

    // TODO: implement process
    /// process the frame according to the frame type
    async fn process_frame(&self, frame: ClushFrame) -> Result<()> {
        match frame.msg_type {
            MessageType::Undefined => Ok(()),
            MessageType::UserMessage => self.process_user_msg(frame).await,
            _ => panic!("unimplemented!"),
        }
    }

    /// process a ClushFrame as user message
    async fn process_user_msg(&self, frame: ClushFrame) -> Result<()> {
        // use auto-generated id
        let id = None;
        // get info from frame
        let from_id = Some(frame.from_id);
        let to_id = Some(frame.to_id);
        let date_time = Some(chrono::Utc::now());
        let content = Some(String::from_utf8(frame.content.to_vec()).unwrap());

        // store UserMsg into database
        let user_msg = UserMsg {
            id,
            from_id,
            to_id,
            date_time,
            content,
        };
        self.db.save::<UserMsg>("", &user_msg).await.unwrap();

        if let Err(_e) = self.tx.send(frame).await {
            panic!("error occurred while sending message")
        }

        Ok(())
    }
}

/// message handler
struct MessageHandler {
    rx: mpsc::Receiver<ClushFrame>,
    map: Arc<DashMap<u64, Task>>,
}

impl MessageHandler {
    /// create a new MessageHandler
    ///
    /// # Example
    ///
    /// ```
    /// let (tx, rx) = mpsc::channel(1024);
    /// let map = Arc::new(DashMap::new());
    /// let handler = MessageHandler::new(rx, map.clone());
    /// ```
    fn new(rx: mpsc::Receiver<ClushFrame>, map: Arc<DashMap<u64, Task>>) -> MessageHandler {
        MessageHandler { rx, map }
    }

    /// handle a frame of user message
    async fn handle_user_msg(&self, frame: ClushFrame) {
        // get (K, V) pair from DashMap, None if not exists
        if let Some(mut pair) = self.map.get_mut(&frame.to_id) {
            // get task from pair
            let task = pair.deref_mut();
            // do write frame
            task.write_frame(frame).await.unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clush_frame_test() {
        let frame = ClushFrame::new(MessageType::UserMessage, 0, 0, 5, BytesMut::from("hello"));
        assert_eq!(0, frame.from_id);
        assert_eq!(0, frame.to_id);
        assert_eq!(5, frame.size);
        assert_eq!(BytesMut::from("hello"), frame.content);
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
        let mut frame = ClushFrame::new(MessageType::UserMessage, 0, 0, 0, BytesMut::from("hello"));
        frame.update_size();
        assert_eq!(5, frame.size);
    }
}
