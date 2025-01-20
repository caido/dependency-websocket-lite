#![warn(clippy::pedantic)]
#![allow(clippy::unused_async)]
#![warn(missing_docs)]

//! A WebSocket server implementation on hyper and websocket-lite.

use std::future::Future;

use http_body_util::Empty;
use hyper::body::{Bytes, Incoming};
use hyper::header::HeaderValue;
use hyper::upgrade::Upgraded;
use hyper::{header, Request, Response, StatusCode};
use hyper_util::rt::TokioIo;
use tokio::task;
use tokio_util::codec::{Decoder, Framed};
use websocket_codec::{ClientRequest, MessageCodec};

pub use websocket_codec::Result;

/// Exposes a `Sink` and a `Stream` for sending and receiving WebSocket messages asynchronously.
pub type AsyncClient = Framed<TokioIo<Upgraded>, MessageCodec>;

/// Accepts a client's WebSocket Upgrade request.
///
/// # Errors
///
/// This method fails when a header required for the WebSocket protocol is missing in the request.
pub async fn server_upgrade<OnClient, F>(
    request: Request<Incoming>,
    on_client: OnClient,
) -> Result<Response<Empty<Bytes>>>
where
    OnClient: FnOnce(AsyncClient) -> F + Send + 'static,
    F: Future<Output = ()> + Send,
{
    let mut response = Response::new(Empty::new());

    let ws_accept = if let Ok(req) = ClientRequest::parse(|name| {
        let h = request.headers().get(name)?;
        h.to_str().ok()
    }) {
        req.ws_accept()
    } else {
        *response.status_mut() = StatusCode::BAD_REQUEST;
        return Ok(response);
    };

    task::spawn(async move {
        match hyper::upgrade::on(request).await {
            Ok(upgraded) => {
                let client = MessageCodec::server().framed(TokioIo::new(upgraded));
                on_client(client).await;
            }
            Err(e) => log::error!("upgrade error: {e}"),
        }
    });

    *response.status_mut() = StatusCode::SWITCHING_PROTOCOLS;

    let headers = response.headers_mut();
    headers.insert(header::UPGRADE, HeaderValue::from_static("websocket"));
    headers.insert(header::CONNECTION, HeaderValue::from_static("Upgrade"));
    headers.insert(header::SEC_WEBSOCKET_ACCEPT, HeaderValue::from_str(&ws_accept)?);
    Ok(response)
}
