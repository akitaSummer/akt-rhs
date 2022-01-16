use std::future::Future;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Semaphore;
use tokio::time::{self, Duration};
use tracing::{debug, error, info};

use crate::types::Result;

pub struct HttpServer {
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
}

impl HttpServer {
    pub async fn run(&self) -> Result<()> {
        loop {
            self.limit_connections.acquire().await.unwrap().forget();

            let socket = self.accept().await?;
        }
    }

    async fn accept(&self) -> Result<TcpStream> {
        let mut try_time = 1;

        loop {
            match self.listener.accept().await {
                Ok((socket, _)) => return Ok(socket),
                Err(err) => {
                    if try_time > 64 {
                        return Err(err.into());
                    }
                }
            }

            time::sleep(Duration::from_secs(try_time)).await;

            try_time *= 2
        }
    }
}

pub async fn run<T: Future>(listener: TcpListener, shutdown: T) {
    let server = HttpServer {
        listener,
        limit_connections: Arc::new(Semaphore::new(500)),
    };

    tokio::select! {
        res = server.run() => {
            if let Err(err) = res {
                error!(cause = %err, "failed to accept");
            }
        }
        _ = shutdown => {
            info!("shutting down");
        }
    };
}
