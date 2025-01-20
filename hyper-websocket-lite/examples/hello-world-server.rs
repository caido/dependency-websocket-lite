#![warn(clippy::pedantic)]

use std::env;

use futures_util::SinkExt;
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_websocket_lite::{server_upgrade, AsyncClient};
use log::LevelFilter;
use simplelog::{Config, SimpleLogger};
use tokio::{net::TcpListener, task::JoinSet};
use websocket_codec::{Message, Result};

async fn on_client(mut client: AsyncClient) {
    let _ = client.send(Message::text("Hello, world!")).await;
    let _ = client.send(Message::close()).await;
}

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::init(LevelFilter::Info, Config::default())?;
    let port = env::args().nth(1).unwrap_or_else(|| "9001".to_owned());
    let addr = format!("0.0.0.0:{port}");
    let listener = TcpListener::bind(addr).await?;

    let mut join_set = JoinSet::new();
    loop {
        let (stream, _addr) = match listener.accept().await {
            Ok(x) => x,
            Err(e) => {
                log::error!("failed to accept connection: {e}");
                continue;
            }
        };

        let serve_connection = async move {
            let result = auto::Builder::new(TokioExecutor::new())
                .serve_connection_with_upgrades(TokioIo::new(stream), service_fn(|req| server_upgrade(req, on_client)))
                .await;

            if let Err(e) = result {
                log::error!("error serving: {e}");
            }
        };

        join_set.spawn(serve_connection);
    }
}
