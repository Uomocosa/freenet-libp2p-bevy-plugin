use crate::identity;
use crate::testing;

#[must_use]
pub fn live_peer_addresses(
    orchestrator: &testing::Orchestrator,
    running: &[testing::RunningPeer],
) -> Vec<testing::LivePeer> {
    let mut live: Vec<testing::LivePeer> = Vec::new();
    for (index, slot) in running.iter().enumerate() {
        let Some(peer) = orchestrator.peers.get(index) else {
            continue;
        };
        let text = std::fs::read_to_string(&slot.log).unwrap_or_default();
        for line in text.lines() {
            if let Some((id, addrs)) = testing::Orchestrator::parse_ready_line(line) {
                let found = identity::Peer { id, addrs };
                if let Some(entry) = live.iter_mut().find(|entry| entry.name == peer.name) {
                    entry.peer = found;
                } else {
                    live.push(testing::LivePeer {
                        name: peer.name.clone(),
                        peer: found,
                    });
                }
            }
        }
    }
    live
}

#[cfg(test)]
mod tests {
    use super::live_peer_addresses;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().expect("tempdir");
        let log = dir.path().join("instance-alice.log");
        std::fs::write(
            &log,
            "lobby freenet node started ws_port=1\np2p: ready peer_id=alice-id addrs=[PeerAddr(\"/ip4/0.0.0.0/tcp/5001\")]\n",
        )
        .expect("write");
        let mut orchestrator = testing::Orchestrator::from_game_and_network(
            "starboard".to_string(),
            testing::Network::DirectDial,
        );
        orchestrator.peer("alice");
        let running = [testing::RunningPeer {
            log,
            child: None,
            exit: None,
        }];
        let live = live_peer_addresses(&orchestrator, &running);
        assert_eq!(live.len(), 1);
        assert_eq!(live[0].name, "alice");
        assert_eq!(live[0].peer.id, identity::PeerId::from("alice-id"));
        assert_eq!(
            live[0].peer.addrs,
            vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/5001")]
        );
    }
}
