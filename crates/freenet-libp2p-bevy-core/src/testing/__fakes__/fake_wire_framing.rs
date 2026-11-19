use atomic_delegate_macros::atomic_delegates;

use crate::testing;

#[derive(Debug, Clone, Copy)]
pub struct FakeWireFraming;

#[atomic_delegates]
impl FakeWireFraming {
    pub async fn write_frame<T>(
        &self,
        writer: &mut tokio::net::tcp::OwnedWriteHalf,
        frame: &T,
    ) -> bool
    where
        T: serde::Serialize + Sync,
    {
    }
    #[must_use]
    pub fn spawn_server_frame_reader(
        self,
        reader: tokio::net::tcp::OwnedReadHalf,
    ) -> tokio::sync::mpsc::UnboundedReceiver<testing::FakeWireServerFrame> {
    }
}

#[cfg(test)]
mod tests {
    use tokio::io::AsyncReadExt;

    use super::FakeWireFraming;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let framing = FakeWireFraming;
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
            .await
            .expect("bind");
        let port = listener.local_addr().expect("port").port();
        let mut client = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .expect("connect");
        let (server, _) = listener.accept().await.expect("accept");
        let (_, mut writer) = server.into_split();
        let frame = testing::FakeWireServerFrame::Extension(vec![1, 2, 3]);
        assert!(framing.write_frame(&mut writer, &frame).await);
        let length = client.read_u32().await.expect("length");
        let mut body = vec![0u8; usize::try_from(length).expect("length")];
        client.read_exact(&mut body).await.expect("body");
        assert_eq!(
            bincode::deserialize::<testing::FakeWireServerFrame>(&body).expect("decode"),
            frame
        );
    }
}
