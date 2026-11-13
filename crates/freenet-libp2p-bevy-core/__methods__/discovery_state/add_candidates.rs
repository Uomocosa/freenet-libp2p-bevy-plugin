use crate::discovery;
use crate::identity;
use crate::time;

pub fn add_candidates(
    state: &mut discovery::state_machine::DiscoveryState,
    peers: impl IntoIterator<Item = identity::Peer>,
    now: time::UnixTime,
) {
    for peer in peers {
        if peer.id == state.me.id || peer.id.is_empty() || peer.addrs.is_empty() {
            continue;
        }
        state.candidates.insert(
            peer.id,
            discovery::state_machine::Candidate {
                addrs: peer.addrs,
                seen_at: now,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::add_candidates;
    use crate::discovery;
    use crate::identity;
    use crate::time;

    fn peer(name: &str, addrs: Vec<identity::PeerAddr>) -> identity::Peer {
        identity::Peer {
            id: identity::PeerId::from(name),
            addrs,
        }
    }

    #[test]
    fn test_usage() {
        let addr = identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001");
        let mut state =
            discovery::state_machine::DiscoveryState::new(peer("me", vec![addr.clone()]));
        add_candidates(
            &mut state,
            vec![
                peer("me", vec![addr.clone()]),
                peer("ghost", Vec::new()),
                peer("alice", vec![addr]),
            ],
            time::UnixTime::from_secs(1),
        );
        assert_eq!(state.candidates.len(), 1);
        assert!(
            state
                .candidates
                .contains_key(&identity::PeerId::from("alice"))
        );
    }
}
