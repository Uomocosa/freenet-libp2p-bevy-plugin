use crate::identity;
use crate::testing;

pub async fn publish_ready_peers(
    orchestrator: &mut testing::Orchestrator,
    running: &[testing::RunningPeer],
    token: &str,
) {
    let Some(board) = orchestrator.board.clone() else {
        return;
    };
    if orchestrator.placement.is_empty() {
        return;
    }
    let mut local: Vec<testing::LivePeer> = Vec::new();
    for peer in &orchestrator.peers {
        if !testing::Orchestrator::peer_runs_here(
            &peer.name,
            &orchestrator.machine,
            &orchestrator.placement,
        ) {
            continue;
        }
        let Ok(id) = orchestrator.peer_id_for_name(token, &peer.name) else {
            continue;
        };
        if local.iter().any(|known| known.name == peer.name) {
            continue;
        }
        local.push(testing::LivePeer {
            name: peer.name.clone(),
            peer: identity::Peer {
                id,
                addrs: Vec::new(),
            },
        });
    }
    let live = orchestrator.live_peer_addresses(running);
    for entry in &live {
        if !testing::Orchestrator::peer_runs_here(
            &entry.name,
            &orchestrator.machine,
            &orchestrator.placement,
        ) {
            continue;
        }
        let routed = testing::Orchestrator::routable_board_peer(entry);
        if let Some(slot) = local.iter_mut().find(|known| known.name == entry.name) {
            slot.peer = routed;
        }
    }
    for entry in &local {
        let signature = entry
            .peer
            .addrs
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(",");
        if orchestrator
            .published_board_peers
            .get(&entry.name)
            .is_some_and(|known| known == &signature)
        {
            continue;
        }
        if board.publish_peer(&entry.name, &entry.peer).await.is_ok() {
            orchestrator
                .published_board_peers
                .insert(entry.name.clone(), signature);
        }
    }
    let mut checkpoints: Vec<testing::ScriptCheckpoint> = Vec::new();
    for peer in &orchestrator.peers {
        if !testing::Orchestrator::peer_runs_here(
            &peer.name,
            &orchestrator.machine,
            &orchestrator.placement,
        ) {
            continue;
        }
        if let Some(done) = peer.starts_after
            && !checkpoints.contains(&done)
        {
            checkpoints.push(done);
        }
    }
    for done in &checkpoints {
        if orchestrator
            .marked_board_checkpoints
            .contains(&done.to_string())
        {
            continue;
        }
        let needle = format!("{}{done}", testing::SCRIPT_CHECKPOINT_COMPLETED_PREFIX);
        let seen = running.iter().any(|slot| {
            std::fs::read_to_string(&slot.log)
                .unwrap_or_default()
                .contains(&needle)
        });
        if seen && board.mark_completed(done).await.is_ok() {
            orchestrator
                .marked_board_checkpoints
                .insert(done.to_string());
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;

    use super::publish_ready_peers;
    use crate::identity;
    use crate::testing;

    struct RecordingBoard {
        published: Mutex<Vec<PublishedPeer>>,
        marked: Mutex<Vec<String>>,
    }

    struct PublishedPeer {
        name: String,
        peer: identity::Peer,
    }

    #[async_trait]
    impl testing::traits::CheckpointBoard for RecordingBoard {
        async fn mark_completed(
            &self,
            checkpoint: &testing::ScriptCheckpoint,
        ) -> Result<(), String> {
            self.marked
                .lock()
                .expect("mark")
                .push(checkpoint.to_string());
            Ok(())
        }

        async fn has_completed(
            &self,
            _checkpoint: &testing::ScriptCheckpoint,
        ) -> Result<bool, String> {
            Ok(false)
        }

        async fn publish_peer(&self, name: &str, peer: &identity::Peer) -> Result<(), String> {
            self.published.lock().expect("publish").push(PublishedPeer {
                name: name.to_string(),
                peer: peer.clone(),
            });
            Ok(())
        }

        async fn find_peer(&self, _name: &str) -> Result<Option<identity::Peer>, String> {
            Ok(None)
        }
    }

    fn orchestrator(board: RecordingBoard) -> testing::Orchestrator {
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        );
        orchestrator.peer("alice");
        orchestrator.set_checkpoint_board(std::sync::Arc::new(board));
        orchestrator
    }

    fn placed(board: std::sync::Arc<RecordingBoard>) -> testing::Orchestrator {
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        );
        orchestrator.peer("alice");
        orchestrator.machine = "a".to_string();
        orchestrator.placement = vec![
            ("alice".to_string(), "a".to_string()),
            ("bob".to_string(), "b".to_string()),
        ];
        orchestrator.set_checkpoint_board(board);
        orchestrator
    }

    #[tokio::test]
    async fn test_usage() {
        let raw = std::sync::Arc::new(RecordingBoard {
            published: Mutex::new(Vec::new()),
            marked: Mutex::new(Vec::new()),
        });
        let mut orchestrator = placed(raw.clone());
        publish_ready_peers(&mut orchestrator, &[], "split-token").await;
        let published = raw.published.lock().expect("published");
        assert_eq!(published.len(), 1);
        assert_eq!(published[0].name, "alice");
        assert_eq!(published[0].peer.addrs.len(), 0);
        let id = orchestrator
            .peer_id_for_name("split-token", "alice")
            .expect("identity");
        assert_eq!(published[0].peer.id, id);
        std::fs::remove_file(orchestrator.peer_identity_file("split-token", "alice"))
            .expect("cleanup");
    }

    #[tokio::test]
    async fn without_a_board_nothing_is_published() {
        let mut orchestrator = orchestrator(RecordingBoard {
            published: Mutex::new(Vec::new()),
            marked: Mutex::new(Vec::new()),
        });
        orchestrator.board = None;
        publish_ready_peers(&mut orchestrator, &[], "split-token").await;
        assert!(orchestrator.published_board_peers.is_empty());
    }
}
