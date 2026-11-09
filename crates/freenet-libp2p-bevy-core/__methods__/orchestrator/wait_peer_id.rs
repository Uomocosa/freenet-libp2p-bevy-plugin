use crate::identity;
use crate::testing;

#[must_use]
pub fn wait_peer_id(
    orchestrator: &testing::Orchestrator,
    name: &str,
    token: &str,
    live: &[testing::LivePeer],
) -> Option<identity::PeerId> {
    if let Some(entry) = live.iter().find(|entry| entry.name == name) {
        return Some(entry.peer.id.clone());
    }
    if orchestrator.is_remote_peer(name) {
        return None;
    }
    orchestrator.peer_id_for_name(token, name).ok()
}

#[cfg(test)]
mod tests {
    use super::wait_peer_id;
    use crate::identity;
    use crate::testing;

    fn live() -> Vec<testing::LivePeer> {
        vec![testing::LivePeer {
            name: "bob".to_string(),
            peer: identity::Peer {
                id: identity::PeerId::from("bob-id"),
                addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001")],
            },
        }]
    }

    fn orchestrator() -> testing::Orchestrator {
        testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        )
    }

    fn placed() -> testing::Orchestrator {
        let mut orchestrator = orchestrator();
        orchestrator.machine = "a".to_string();
        orchestrator.placement = vec![
            ("alice".to_string(), "a".to_string()),
            ("bob".to_string(), "b".to_string()),
        ];
        orchestrator
    }

    #[test]
    fn test_usage() {
        let found = wait_peer_id(&orchestrator(), "bob", "token", &live()).expect("live");
        assert_eq!(found, identity::PeerId::from("bob-id"));
        let made = wait_peer_id(&orchestrator(), "mallory", "wait-token", &[]).expect("stable id");
        let again = wait_peer_id(&orchestrator(), "mallory", "wait-token", &[]).expect("stable id");
        assert_eq!(made, again);
        std::fs::remove_file(orchestrator().peer_identity_file("wait-token", "mallory"))
            .expect("cleanup");
    }

    #[test]
    fn remote_peers_defer_until_the_board_knows_them() {
        let orchestrator = placed();
        assert_eq!(wait_peer_id(&orchestrator, "bob", "token", &[]), None);
        let id = identity::PeerId::from("board-bob");
        let remote = vec![testing::LivePeer {
            name: "bob".to_string(),
            peer: identity::Peer {
                id: id.clone(),
                addrs: Vec::new(),
            },
        }];
        assert_eq!(
            wait_peer_id(&orchestrator, "bob", "token", &remote),
            Some(id)
        );
    }
}
