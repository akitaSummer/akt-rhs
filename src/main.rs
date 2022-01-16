mod http;
mod types;

use tokio::net::TcpListener;
use tokio::signal;
use tokio::spawn;

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("localhost:8888").await.unwrap();

    spawn(async move {
        http::run(listener, signal::ctrl_c()).await;
    });

    println!("Hello, world!");
}
