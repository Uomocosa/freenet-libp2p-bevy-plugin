use crate::testing;

pub async fn remote_board_peers(
    orchestrator: &testing::Orchestrator,
    running: &[testing::RunningPeer],
) -> Vec<testing::LivePeer> {
    let Some(board) = orchestrator.board.clone() else {
        return Vec::new();
    };
    if orchestrator.placement.is_empty() {
        return Vec::new();
    }
    let waiting = orchestrator.peers.iter().enumerate().any(|(index, peer)| {
        testing::Orchestrator::peer_runs_here(
            &peer.name,
            &orchestrator.machine,
            &orchestrator.placement,
        ) && running.get(index).is_none_or(|slot| slot.child.is_none())
    });
    if !waiting {
        return Vec::new();
    }
    let mut remote: Vec<testing::LivePeer> = Vec::new();
    for peer in &orchestrator.peers {
        if testing::Orchestrator::peer_runs_here(
            &peer.name,
            &orchestrator.machine,
            &orchestrator.placement,
        ) {
            continue;
        }
        if remote
            .iter()
            .any(|entry: &testing::LivePeer| entry.name == peer.name)
        {
            continue;
        }
        if let Ok(Some(found)) = board.find_peer(&peer.name).await {
            remote.push(testing::LivePeer {
                name: peer.name.clone(),
                peer: found,
            });
        }
    }
    remote
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;

    use super::remote_board_peers;
    use crate::identity;
    use crate::testing;

    struct StubBoard {
        queries: Mutex<Vec<String>>,
        peer: Option<identity::Peer>,
    }

    #[async_trait]
    impl testing::CheckpointBoard for StubBoard {
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
            Ok(false)
        }

        async fn publish_peer(&self, _name: &str, _peer: &identity::Peer) -> Result<(), String> {
            Ok(())
        }

        async fn find_peer(&self, name: &str) -> Result<Option<identity::Peer>, String> {
            self.queries.lock().expect("queries").push(name.to_string());
            Ok(self.peer.clone())
        }
    }

    fn orchestrator(board: std::sync::Arc<StubBoard>) -> testing::Orchestrator {
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        );
        orchestrator.peer("alice");
        orchestrator.peer("bob");
        orchestrator.machine = "a".to_string();
        orchestrator.placement = vec![
            ("alice".to_string(), "a".to_string()),
            ("bob".to_string(), "b".to_string()),
        ];
        orchestrator.set_checkpoint_board(board);
        orchestrator
    }

    fn unplaced(board: std::sync::Arc<StubBoard>) -> testing::Orchestrator {
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        );
        orchestrator.peer("alice");
        orchestrator.peer("bob");
        orchestrator.set_checkpoint_board(board);
        orchestrator
    }

    fn stub(peer: Option<identity::Peer>) -> std::sync::Arc<StubBoard> {
        std::sync::Arc::new(StubBoard {
            queries: Mutex::new(Vec::new()),
            peer,
        })
    }

    fn bob() -> identity::Peer {
        identity::Peer {
            id: identity::PeerId::from("bob-id"),
            addrs: vec![identity::PeerAddr::from("/ip4/192.168.1.20/tcp/5001")],
        }
    }

    #[tokio::test]
    async fn test_usage() {
        let board = stub(Some(bob()));
        let orchestrator = orchestrator(board.clone());
        let remote = remote_board_peers(&orchestrator, &[]).await;
        assert_eq!(remote.len(), 1);
        assert_eq!(remote[0].name, "bob");
        assert_eq!(remote[0].peer.id, identity::PeerId::from("bob-id"));
        assert_eq!(board.queries.lock().expect("queries").as_slice(), ["bob"]);
    }

    #[tokio::test]
    async fn without_a_split_no_board_read_happens() {
        let board = stub(Some(bob()));
        let orchestrator = unplaced(board.clone());
        let remote = remote_board_peers(&orchestrator, &[]).await;
        assert_eq!(remote.len(), 0);
        assert!(board.queries.lock().expect("queries").is_empty());
    }
}
