use crate::discovery;
use crate::identity;
use discovery::state_machine::State;

pub fn add_candidates(
    state: &mut State,
    peers: impl IntoIterator<Item = identity::Peer>,
    now: discovery::UnixTime,
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
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let peers = vec![
            identity::Peer {
                id: identity::PeerId::from("me"),
                addrs: vec![identity::PeerAddr::from("/ip4/1")],
            },
            identity::Peer {
                id: identity::PeerId::from("a"),
                addrs: vec![identity::PeerAddr::from("/ip4/2")],
            },
            identity::Peer {
                id: identity::PeerId::from("b"),
                addrs: Vec::new(),
            },
        ];
        add_candidates(&mut state, peers, discovery::UnixTime::from_secs(1));
        assert_eq!(state.candidates.len(), 1);
        assert!(state.candidates.contains_key(&identity::PeerId::from("a")));
    }
}
