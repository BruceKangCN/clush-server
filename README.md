# clush-server

a rust based server application for an IM software  

**still under early development**  

the client is written in `C++/Qt` called [`clush-client`](https://github.com/BruceKangCN/clush-client)  

## Build&Run

1. install `git`, `rustup`  
2. install `rust toolchain` through `rustup`  
    ```
    rustup toolchain install stable
    ```
3. checkout the repository and build with `git` & `cargo`  
    ```
    git clone https://github.com/BruceKangCN/clush-server.git
    cd clush-server
    cargo build --release
    ```
4. run with `cargo`  
    ```
    cargo run --release
    ```

## Config

the  server needs a configuration file in `config/clush.json` to start  
here is an example  
```json
{
    "server_config": {
        "url": "0.0.0.0:9527",
        "enable_tls": false,
        "key_path": "config/key.pem",
        "cert_path": "config/certificate.pem"
    },
    "rbatis_config": {
        "db_url": "postgres://root:root@example.com/test",
        "log_path": "log/rbatis.log",
        "log_level": "Warn",
        "log_limit": 10000,
        "debug_mode": false
    }
}
```

## Credits

### MIT

* tokio
* tokio-rustls
* bytes
* serde
* serde_json
* chrono
* log
* fast_log
* sha2

### Apache 2.0

* rbatis
