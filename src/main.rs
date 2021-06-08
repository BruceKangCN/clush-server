//! # clush-server
//!
//! the server of clush, a cross-platform IM software
//!
//! MIT License
//! Copyright (c) 2021 Bruce Kang

#[macro_use]
extern crate rbatis;

pub mod config;
pub mod entity;
pub mod util;

mod core;

use crate::config::ClushConfig;
use crate::core::ClushServer;
use tokio::io::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let config = ClushConfig::from_json("config/clush.json").await;
    let server = ClushServer::init_with_config(config).await?;
    server.start().await
}
