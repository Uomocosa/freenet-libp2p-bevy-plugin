use thiserror::Error;

use crate::discovery;

#[derive(Error, Debug)]
pub enum Error {
    #[error("connection timed out")]
    ConnectionTimeout,
    #[error("freenet {0} timed out")]
    Timeout(discovery::FreenetRequestKind),
    #[error("disconnected")]
    Disconnected,
    #[error("websocket error")]
    WebSocket(#[source] Box<tokio_tungstenite::tungstenite::Error>),
    #[error("serialization error")]
    Serialization(#[from] bincode::Error),
    #[error("http error")]
    Http(#[from] http::Error),
    #[error("io error")]
    Io(#[from] std::io::Error),
    #[error("channel send error")]
    ChannelSend,
    #[error("contract not found")]
    ContractNotFound,
    #[error("freenet stdlib error")]
    Client(#[from] freenet_stdlib::client_api::ClientError),
    #[error("unexpected response: {0}")]
    UnexpectedResponse(String),
    #[error("room size must be non-zero")]
    ZeroRoomSize,
}

#[rustfmt::skip]
impl From<tokio_tungstenite::tungstenite::Error> for Error {
    fn from(error: tokio_tungstenite::tungstenite::Error) -> Self { Self::WebSocket(Box::new(error)) }
}

// no test_usage necessary
