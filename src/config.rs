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
