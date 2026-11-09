use tokio::io::AsyncReadExt;

use crate::testing;

#[must_use]
pub fn spawn_server_frame_reader(
    framing: &testing::FakeWireFraming,
    mut reader: tokio::net::tcp::OwnedReadHalf,
) -> tokio::sync::mpsc::UnboundedReceiver<testing::FakeWireServerFrame> {
    let _ = framing;
    let (frame_tx, frame_rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(async move {
        loop {
            let Ok(length) = reader.read_u32().await else {
                tracing::error!(target: "testing", "fake wire frame length read failed");
                break;
            };
            let length = usize::try_from(length).unwrap_or(0);
            let mut body = vec![0u8; length];
            if reader.read_exact(&mut body).await.is_err() {
                tracing::error!(target: "testing", "fake wire frame body read failed");
                break;
            }
            let Ok(frame) = bincode::deserialize::<testing::FakeWireServerFrame>(&body) else {
                tracing::error!(target: "testing", "fake wire frame decode failed");
                break;
            };
            if frame_tx.send(frame).is_err() {
                break;
            }
        }
    });
    frame_rx
}

#[cfg(test)]
mod tests {
    use super::spawn_server_frame_reader;
    use crate::discovery;
    use crate::testing;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn split_frame_arrives_intact_while_other_branch_is_busy() {
        let framing = testing::FakeWireFraming;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let port = listener.local_addr().expect("port").port();
        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (_, mut writer) = stream.into_split();
            let frame = testing::FakeWireServerFrame::Lobby(discovery::Lobby::default());
            let bytes = bincode::serialize(&frame).expect("encode");
            let length = u32::try_from(bytes.len()).expect("length");
            writer.write_u32(length).await.expect("length");
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            writer.write_all(&bytes).await.expect("body");
        });
        let stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .expect("connect");
        let (reader, _) = stream.into_split();
        let mut frames = spawn_server_frame_reader(&framing, reader);
        let mut ticks: u32 = 0;
        let frame = loop {
            tokio::select! {
                () = tokio::time::sleep(std::time::Duration::from_millis(5)) => {
                    ticks = ticks.saturating_add(1);
                }
                incoming = frames.recv() => {
                    break incoming;
                }
            }
        };
        assert!(
            frame.is_some(),
            "split frame must arrive intact, ticks={ticks}"
        );
        server.abort();
    }

    #[tokio::test]
    async fn test_usage() {
        let framing = testing::FakeWireFraming;
        let (_tx, rx) = tokio::sync::mpsc::unbounded_channel::<testing::FakeWireServerFrame>();
        drop(rx);
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let port = listener.local_addr().expect("port").port();
        let stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .expect("connect");
        let (reader, _) = stream.into_split();
        let _ = framing.spawn_server_frame_reader(reader);
    }
}
