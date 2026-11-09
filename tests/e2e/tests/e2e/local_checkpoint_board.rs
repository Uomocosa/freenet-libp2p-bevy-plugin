use std::path::PathBuf;

use async_trait::async_trait;
use freenet_libp2p_bevy_plugin::{identity, testing};

pub struct LocalCheckpointBoard {
    dir: PathBuf,
}

impl LocalCheckpointBoard {
    pub const fn new(dir: PathBuf) -> Self {
        Self { dir }
    }

    pub fn board_dir_from_env() -> PathBuf {
        let raw = std::env::var("E2E_BOARD_DIR").unwrap_or_default();
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
        std::env::temp_dir().join(format!("e2e-board-{}", std::process::id()))
    }

    fn checkpoint_path(&self, checkpoint: &testing::ScriptCheckpoint) -> PathBuf {
        self.dir.join(format!("checkpoint-{checkpoint}"))
    }

    fn peer_path(&self, name: &str) -> PathBuf {
        self.dir.join(format!("peer-{name}.json"))
    }

    fn ensure_dir(&self) -> Result<(), String> {
        std::fs::create_dir_all(&self.dir)
            .map_err(|error| format!("board dir {}: {error}", self.dir.display()))
    }
}

#[async_trait]
impl testing::CheckpointBoard for LocalCheckpointBoard {
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
        let path = self.peer_path(name);
        if !path.exists() {
            return Ok(None);
        }
        let text =
            std::fs::read_to_string(&path).map_err(|error| format!("read peer {name}: {error}"))?;
        serde_json::from_str(&text)
            .map(Some)
            .map_err(|error| format!("decode peer {name}: {error}"))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::LocalCheckpointBoard;
    use freenet_libp2p_bevy_plugin::testing::CheckpointBoard;
    use freenet_libp2p_bevy_plugin::{identity, testing};

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
