use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// all configuration
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClushConfig {
    #[serde(default = "ClushConfig::default_server_config")]
    pub server_config: ServerConfig,
    #[serde(default = "ClushConfig::default_rbatis_config")]
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

    fn default_server_config() -> ServerConfig {
        let url = ServerConfig::default_url();
        let enable_tls = ServerConfig::default_enable_tls();
        let key_path = ServerConfig::default_key_path();
        let cert_path = ServerConfig::default_cert_path();

        ServerConfig {
            url,
            enable_tls,
            key_path,
            cert_path,
        }
    }

    fn default_rbatis_config() -> RbatisConfig {
        let db_url = RbatisConfig::default_db_url();
        let log_path = RbatisConfig::default_log_path();
        let log_level = RbatisConfig::default_log_level();
        let log_limit = RbatisConfig::default_log_limit();
        let debug_mode = RbatisConfig::default_debug_mode();

        RbatisConfig {
            db_url,
            log_path,
            log_level,
            log_limit,
            debug_mode,
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerConfig {
    #[serde(default = "ServerConfig::default_url")]
    pub url: String,
    #[serde(default = "ServerConfig::default_enable_tls")]
    pub enable_tls: bool,
    #[serde(default = "ServerConfig::default_key_path")]
    pub key_path: String,
    #[serde(default = "ServerConfig::default_cert_path")]
    pub cert_path: String,
}

impl ServerConfig {
    fn default_url() -> String {
        "0.0.0.0:9527".to_string()
    }

    fn default_enable_tls() -> bool {
        false
    }

    fn default_key_path() -> String {
        "config/key.pem".to_string()
    }

    fn default_cert_path() -> String {
        "config/cert.pem".to_string()
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RbatisConfig {
    #[serde(default = "RbatisConfig::default_db_url")]
    pub db_url: String,
    #[serde(default = "RbatisConfig::default_log_path")]
    pub log_path: String,
    #[serde(default = "RbatisConfig::default_log_level")]
    pub log_level: String,
    #[serde(default = "RbatisConfig::default_log_limit")]
    pub log_limit: usize,
    #[serde(default = "RbatisConfig::default_debug_mode")]
    pub debug_mode: bool,
}

impl RbatisConfig {
    pub fn log_level(&self) -> log::Level {
        log::Level::from_str(&self.log_level).unwrap()
    }

    fn default_db_url() -> String {
        "postgres://root:root@127.0.0.1/test".to_string()
    }

    fn default_log_path() -> String {
        "log/rbatis.log".to_string()
    }

    fn default_log_level() -> String {
        "Warn".to_string()
    }

    fn default_log_limit() -> usize {
        10000usize
    }

    fn default_debug_mode() -> bool {
        false
    }
}
