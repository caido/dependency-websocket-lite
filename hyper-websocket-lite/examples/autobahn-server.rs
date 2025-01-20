#![warn(clippy::pedantic)]

use futures_util::{SinkExt, StreamExt};
use hyper::service::service_fn;
use hyper_util::rt::{TokioExecutor, TokioIo};
use hyper_util::server::conn::auto;
use hyper_websocket_lite::{server_upgrade, AsyncClient};
use log::LevelFilter;
use simplelog::{Config, SimpleLogger};
use tokio::{net::TcpListener, task::JoinSet};
use websocket_codec::{Message, Opcode, Result};

async fn on_client(mut stream_mut: AsyncClient) {
    log::info!("On client");

    let mut stream = loop {
        let (msg, mut stream) = stream_mut.into_future().await;

        let msg = match msg {
            Some(Ok(msg)) => msg,
            Some(Err(err)) => {
                log::error!("error receiving message: {err}");
                let _ = stream.send(Message::close()).await;
                break stream;
            }
            None => {
                break stream;
            }
        };

        let _ = match msg.opcode() {
            Opcode::Text | Opcode::Binary => stream.send(msg).await,
            Opcode::Ping => stream.send(Message::pong(msg.into_data())).await,
            Opcode::Close => {
                break stream;
            }
            Opcode::Pong => Ok(()),
        };

        stream_mut = stream;
    };

    let _ = stream.send(Message::close()).await;
}

#[tokio::main]
async fn main() -> Result<()> {
    SimpleLogger::init(LevelFilter::Info, Config::default())?;
    let addr = "127.0.0.1:9001";
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
