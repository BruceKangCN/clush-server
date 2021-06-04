//! # clush-server
//!
//! the server of clush, a cross-platform IM software
//!
//! MIT License
//! Copyright (c) 2021 Bruce Kang

mod server;
mod task;

use crate::server::ClushServer;
use tokio::io::Result;
use std::env;

#[tokio::main]
async fn main() -> Result<()> {
    let server: ClushServer;
    if env::args().len() > 1 {
        server = ClushServer::init_with_addr(env::args().nth(1).unwrap()).await?;
    } else {
        server = ClushServer::init().await?;
    }
    server.start().await
}
