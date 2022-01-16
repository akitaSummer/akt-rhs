pub mod request;
pub mod response;
pub mod shutdown;

use std::future::Future;
use std::sync::Arc;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{broadcast, mpsc, Semaphore};
use tokio::time::{self, Duration};
use tracing::{debug, error, info};

use crate::config::init;
use crate::http::request::HttpRequest;
use crate::http::shutdown::Shutdown;
use crate::router::Router;
use crate::types::Result;

pub struct HttpServer {
    listener: TcpListener,
    limit_connections: Arc<Semaphore>,
    notify_shutdown: broadcast::Sender<()>,
    shutdown_complete_rx: mpsc::Receiver<()>,
    shutdown_complete_tx: mpsc::Sender<()>,
}

impl HttpServer {
    pub async fn run(&self) -> Result<()> {
        loop {
            self.limit_connections.acquire().await.unwrap().forget();

            let socket = self.accept().await?;

            let mut handler = Handler {
                socket: socket,
                limit_connections: self.limit_connections.clone(),
                shutdown: Shutdown::new(self.notify_shutdown.subscribe()),
                _shutdown_complete: self.shutdown_complete_tx.clone(),
            };

            tokio::spawn(async move {
                if let Err(err) = handler.run().await {
                    error!(cause = ?err, "connection error");
                }
            });
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
    init();
    let (notify_shutdown, _) = broadcast::channel(1);
    let (shutdown_complete_tx, shutdown_complete_rx) = mpsc::channel(1);

    let mut server = HttpServer {
        listener,
        limit_connections: Arc::new(Semaphore::new(500)),
        notify_shutdown,
        shutdown_complete_tx,
        shutdown_complete_rx,
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

    drop(server.notify_shutdown);
    drop(server.shutdown_complete_tx);
    let _ = server.shutdown_complete_rx.recv().await;
}

struct Handler {
    socket: TcpStream,
    limit_connections: Arc<Semaphore>,
    shutdown: Shutdown,
    _shutdown_complete: mpsc::Sender<()>,
}

impl Handler {
    async fn run(&mut self) -> Result<()> {
        while !self.shutdown.is_shutdown() {
            let request = HttpRequest::from(&mut self.socket).await;
            println!("request is: {:?}", request.resource);
            let resp = Router::route(&request).await;
            let resp_str: String = resp.into();
            self.socket
                .write(resp_str.as_bytes() as &[u8])
                .await
                .unwrap();
            self.socket.flush().await.unwrap();
        }

        Ok(())
    }
}

impl Drop for Handler {
    fn drop(&mut self) {
        self.limit_connections.add_permits(1);
    }
}
