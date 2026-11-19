use async_trait::async_trait;
use derive_more::Deref;

use crate::identity;
use crate::testing;

#[derive(Deref)]
pub struct StubBoard(pub Option<identity::Peer>);

#[async_trait]
#[rustfmt::skip]
impl testing::traits::CheckpointBoard for StubBoard {
    async fn mark_completed(&self, _checkpoint: &testing::ScriptCheckpoint) -> Result<(), String> {
        Ok(())
    }

    async fn has_completed(&self, _checkpoint: &testing::ScriptCheckpoint) -> Result<bool, String> {
        Ok(true)
    }

    async fn publish_peer(&self, _name: &str, _peer: &identity::Peer) -> Result<(), String> {
        Ok(())
    }

    async fn find_peer(&self, _name: &str) -> Result<Option<identity::Peer>, String> {
        Ok((*self).clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::testing;
    use crate::testing::traits::CheckpointBoard;

    use super::StubBoard;

    #[tokio::test]
    async fn test_usage() {
        let peer = testing::peer("peer-1", &["/ip4/127.0.0.1/tcp/5001"]);
        let board = StubBoard(Some(peer));
        assert!(
            board
                .mark_completed(&testing::ScriptCheckpoint::new("done"))
                .await
                .is_ok()
        );
        assert!(
            board
                .has_completed(&testing::ScriptCheckpoint::new("done"))
                .await
                .is_ok()
        );
        assert!(board.find_peer("alice").await.expect("find").is_some());
        assert!(
            StubBoard(None)
                .find_peer("nobody")
                .await
                .expect("find")
                .is_none()
        );
    }
}
