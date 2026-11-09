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
    use async_trait::async_trait;
    use derive_more::Deref;

    use super::CheckpointBoard;
    use crate::identity;
    use crate::testing;

    #[derive(Deref)]
    struct StubBoard(Option<identity::Peer>);

    #[async_trait]
    impl CheckpointBoard for StubBoard {
        async fn mark_completed(
            &self,
            _checkpoint: &testing::ScriptCheckpoint,
        ) -> Result<(), String> {
            Ok(())
        }

        async fn has_completed(
            &self,
            _checkpoint: &testing::ScriptCheckpoint,
        ) -> Result<bool, String> {
            Ok(true)
        }

        async fn publish_peer(&self, _name: &str, _peer: &identity::Peer) -> Result<(), String> {
            Ok(())
        }

        async fn find_peer(&self, _name: &str) -> Result<Option<identity::Peer>, String> {
            Ok((*self).clone())
        }
    }

    #[tokio::test]
    async fn test_usage() {
        let peer = identity::Peer {
            id: identity::PeerId::from("peer-1"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001")],
        };
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
