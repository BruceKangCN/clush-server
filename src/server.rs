use crate::task::Task;
use tokio::net::TcpListener;
use tokio::io::Result;

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
        ClushServer {
            listener,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
}
