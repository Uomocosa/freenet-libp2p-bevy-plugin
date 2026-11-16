use std::path::PathBuf;

use async_trait::async_trait;
use atomic_delegate_macros::atomic_delegates;
use derive_more::Deref;

use crate::identity;
use crate::testing;

#[derive(Deref)]
pub struct LocalCheckpointBoard(pub PathBuf);

impl LocalCheckpointBoard {
    #[must_use]
    pub const fn new(dir: PathBuf) -> Self {
        Self(dir)
    }
}

#[async_trait]
impl testing::traits::CheckpointBoard for LocalCheckpointBoard {
    async fn mark_completed(&self, checkpoint: &testing::ScriptCheckpoint) -> Result<(), String> {
        self.ensure_dir()?;
        std::fs::write(self.checkpoint_path(checkpoint), "")
            .map_err(|error| format!("mark {checkpoint}: {error}"))
    }

    async fn has_completed(&self, checkpoint: &testing::ScriptCheckpoint) -> Result<bool, String> {
        self.ensure_dir()?;
        Ok(self.checkpoint_path(checkpoint).exists())
    }

    async fn publish_peer(&self, name: &str, peer: &identity::Peer) -> Result<(), String> {
        self.ensure_dir()?;
        let text =
            serde_json::to_string(peer).map_err(|error| format!("encode peer {name}: {error}"))?;
        std::fs::write(self.peer_path(name), text)
            .map_err(|error| format!("publish peer {name}: {error}"))
    }

    async fn find_peer(&self, name: &str) -> Result<Option<identity::Peer>, String> {
        self.ensure_dir()?;
        self.read_peer_file(name).await
    }
}

#[atomic_delegates]
impl LocalCheckpointBoard {
    pub fn board_dir_from_env() -> PathBuf {}
}

#[rustfmt::skip]
impl LocalCheckpointBoard {
    pub(crate) fn checkpoint_path(&self, checkpoint: &testing::ScriptCheckpoint) -> PathBuf { self.join(format!("checkpoint-{checkpoint}")) }
    pub(crate) fn peer_path(&self, name: &str) -> PathBuf { self.join(format!("peer-{name}.json")) }
}

#[atomic_delegates]
impl LocalCheckpointBoard {
    pub(crate) fn ensure_dir(&self) -> Result<(), String> {}
    pub(crate) async fn read_peer_file(
        &self,
        name: &str,
    ) -> Result<Option<identity::Peer>, String> {
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::LocalCheckpointBoard;
    use crate::identity;
    use crate::testing;
    use crate::testing::traits::CheckpointBoard;

    fn board() -> (tempfile::TempDir, LocalCheckpointBoard) {
        let dir = tempfile::tempdir().expect("tempdir");
        let board = LocalCheckpointBoard::new(dir.path().to_path_buf());
        (dir, board)
    }

    fn peer() -> identity::Peer {
        identity::Peer {
            id: identity::PeerId::from("peer-1"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001")],
        }
    }

    #[tokio::test]
    async fn test_usage() {
        let (_dir, board) = board();
        assert!(
            !board
                .has_completed(&testing::ScriptCheckpoint::new("done"))
                .await
                .expect("has")
        );
        board
            .mark_completed(&testing::ScriptCheckpoint::new("done"))
            .await
            .expect("mark");
        assert!(
            board
                .has_completed(&testing::ScriptCheckpoint::new("done"))
                .await
                .expect("has")
        );
        assert!(board.find_peer("alice").await.expect("find").is_none());
        board.publish_peer("alice", &peer()).await.expect("publish");
        assert_eq!(
            board.find_peer("alice").await.expect("find").expect("peer"),
            peer()
        );
    }

    #[tokio::test]
    async fn publish_then_find_returns_the_same_peer() {
        let (_dir, board) = board();
        let original = identity::Peer {
            id: identity::PeerId::from("peer-1"),
            addrs: vec![
                identity::PeerAddr::from("/ip4/192.168.1.20/tcp/51234"),
                identity::PeerAddr::from("/ip4/192.168.1.20/udp/51235/quic-v1"),
            ],
        };
        board
            .publish_peer("alice", &original)
            .await
            .expect("publish");
        assert_eq!(
            board.find_peer("alice").await.expect("find").expect("peer"),
            original
        );
    }

    #[tokio::test]
    async fn checkpoints_do_not_clash_with_peers() {
        let (_dir, board) = board();
        board.publish_peer("done", &peer()).await.expect("publish");
        assert!(
            !board
                .has_completed(&testing::ScriptCheckpoint::new("done"))
                .await
                .expect("has")
        );
    }

    #[test]
    fn board_dir_defaults_outside_artifacts_and_logs() {
        let dir = LocalCheckpointBoard::board_dir_from_env();
        let results = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(".local-run");
        assert!(!dir.starts_with(&results));
        assert_ne!(dir.extension().and_then(|ext| ext.to_str()), Some("log"));
    }
}
