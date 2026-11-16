use crate::identity;
use crate::testing;

pub async fn read_peer_file(
    board: &testing::harness::LocalCheckpointBoard,
    name: &str,
) -> Result<Option<identity::Peer>, String> {
    let path = board.peer_path(name);
    if !path.exists() {
        return Ok(None);
    }
    let text =
        std::fs::read_to_string(&path).map_err(|error| format!("read peer {name}: {error}"))?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|error| format!("decode peer {name}: {error}"))
}

#[cfg(test)]
mod tests {
    use super::read_peer_file;
    use crate::identity;
    use crate::testing;
    use crate::testing::traits::CheckpointBoard;

    #[tokio::test]
    async fn test_usage() {
        let dir = tempfile::tempdir().expect("temp dir");
        let board = testing::harness::LocalCheckpointBoard::new(dir.path().to_path_buf());
        board.ensure_dir().expect("ensure");
        assert!(
            read_peer_file(&board, "alice")
                .await
                .expect("find")
                .is_none()
        );
        let peer = identity::Peer {
            id: identity::PeerId::from("peer-1"),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001")],
        };
        board.publish_peer("alice", &peer).await.expect("publish");
        assert_eq!(
            read_peer_file(&board, "alice")
                .await
                .expect("find")
                .expect("peer"),
            peer
        );
    }
}
