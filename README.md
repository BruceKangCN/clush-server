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
    "serverConfig": {
        "url": "0.0.0.0:9527",
        "enableTls": false,
        "keyPath": "config/key.pem",
        "certPath": "config/certificate.pem"
    },
    "rbatisConfig": {
        "dbUrl": "postgres://root:root@example.com/test",
        "logPath": "log/rbatis.log",
        "logLevel": "Warn",
        "logLimit": 10000,
        "debugMode": false
    }
}
```

## Credits

### Special Thanks

* @rekey

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
