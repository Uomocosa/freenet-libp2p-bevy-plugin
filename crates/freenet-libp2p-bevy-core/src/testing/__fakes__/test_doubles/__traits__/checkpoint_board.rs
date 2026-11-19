use async_trait::async_trait;

use crate::identity;
use crate::testing;

#[async_trait]
pub trait CheckpointBoard: Send + Sync {
    async fn mark_completed(&self, checkpoint: &testing::ScriptCheckpoint) -> Result<(), String>;
    async fn has_completed(&self, checkpoint: &testing::ScriptCheckpoint) -> Result<bool, String>;
    async fn publish_peer(&self, name: &str, peer: &identity::Peer) -> Result<(), String>;
    async fn find_peer(&self, name: &str) -> Result<Option<identity::Peer>, String>;
}

#[cfg(test)]
mod tests {
    use super::CheckpointBoard;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let peer = testing::peer("peer-1", &["/ip4/127.0.0.1/tcp/5001"]);
        let board = testing::StubBoard(Some(peer));
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
            testing::StubBoard(None)
                .find_peer("nobody")
                .await
                .expect("find")
                .is_none()
        );
    }
}
