use tokio::io::AsyncWriteExt;

use crate::testing;

pub async fn write_frame<T>(
    framing: &testing::FakeWireFraming,
    writer: &mut tokio::net::tcp::OwnedWriteHalf,
    frame: &T,
) -> bool
where
    T: serde::Serialize + Sync,
{
    let _ = framing;
    let Ok(bytes) = bincode::serialize(frame) else {
        tracing::error!(target: "testing", "fake wire frame encode failed");
        return true;
    };
    let Ok(length) = u32::try_from(bytes.len()) else {
        tracing::error!(target: "testing", "fake wire frame too large");
        return true;
    };
    if writer.write_u32(length).await.is_err() {
        tracing::error!(target: "testing", "fake wire frame write failed");
        return false;
    }
    if writer.write_all(&bytes).await.is_err() {
        tracing::error!(target: "testing", "fake wire frame write failed");
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use tokio::io::AsyncReadExt;

    use super::write_frame;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let framing = testing::FakeWireFraming;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let port = listener.local_addr().expect("port").port();
        let accept = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let (mut reader, _) = stream.into_split();
            let mut body = vec![0u8; 4];
            reader.read_exact(&mut body).await.expect("read");
        });
        let stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .expect("connect");
        let (_, mut writer) = stream.into_split();
        let frame = testing::FakeWireClientFrame::JoinedRoomBytes(vec![1, 2, 3]);
        assert!(write_frame(&framing, &mut writer, &frame).await);
        accept.abort();
    }
}
