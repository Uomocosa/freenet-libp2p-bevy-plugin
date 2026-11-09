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
        &self,
        reader: tokio::net::tcp::OwnedReadHalf,
    ) -> tokio::sync::mpsc::UnboundedReceiver<testing::FakeWireServerFrame> {
    }
}

#[cfg(test)]
mod tests {
    use super::FakeWireFraming;

    #[test]
    fn test_usage() {
        let _ = FakeWireFraming;
    }
}
