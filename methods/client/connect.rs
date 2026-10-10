use std::time::Duration;

use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::info;

use freenet_stdlib::client_api::{ClientError, HostResponse};

use crate::discovery;

pub async fn connect(
    host: &str,
    port: u16,
) -> Result<discovery::freenet::Client, discovery::Error> {
    let url = format!("ws://{host}:{port}/v1/contract/command?encodingProtocol=native");
    info!(target: "room_lobby", url = %url, "connecting to freenet node");
    let mut request = url.into_client_request()?;
    request.headers_mut().insert(
        "encoding-protocol",
        http::HeaderValue::from_static("native"),
    );
    let ws = tokio::time::timeout(Duration::from_secs(5), connect_async(request));
    let (ws_stream, _) = ws
        .await
        .map_err(|_| discovery::Error::ConnectionTimeout)??;
    let (mut ws_write, mut ws_read) = ws_stream.split();
    let (write_tx, mut write_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
    let (read_tx, read_rx) =
        tokio::sync::mpsc::unbounded_channel::<Result<HostResponse, ClientError>>();
    tokio::spawn(async move {
        while let Some(msg) = write_rx.recv().await {
            if ws_write.send(msg).await.is_err() {
                break;
            }
        }
    });
    let write_tx_clone = write_tx.clone();
    tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_read.next().await {
            match msg {
                Message::Binary(data) => {
                    if let Ok(result) =
                        bincode::deserialize::<Result<HostResponse, ClientError>>(&data)
                    {
                        let _ = read_tx.send(result);
                    }
                }
                Message::Close(frame) => {
                    info!(target: "room_lobby", ?frame, "websocket closed");
                    break;
                }
                Message::Ping(data) => {
                    let _ = write_tx_clone.send(Message::Pong(data));
                }
                _ => {}
            }
        }
    });
    Ok(discovery::freenet::Client {
        write: write_tx,
        read: read_rx,
    })
}

#[cfg(test)]
mod tests {
    use super::connect;

    #[tokio::test]
    async fn test_usage() {
        assert!(connect("127.0.0.1", 1).await.is_err());
    }
}
