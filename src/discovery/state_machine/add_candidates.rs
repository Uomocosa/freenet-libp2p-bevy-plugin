use crate::discovery;
use crate::net_id;
use discovery::state_machine::State;

pub fn add_candidates(
    state: &mut State,
    peers: impl IntoIterator<Item = net_id::Peer>,
    now: discovery::UnixTime,
) {
    for peer in peers {
        if peer.id == state.me.id || peer.id.is_empty() || peer.addrs.is_empty() {
            continue;
        }
        state.candidates.insert(
            peer.id,
            discovery::Presence {
                addrs: peer.addrs,
                updated_at: now,
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::add_candidates;
    use crate::discovery;
    use crate::net_id;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: Vec::new(),
            },
            discovery::Timing::default(),
        );
        let peers = vec![
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: vec![net_id::PeerAddr::from("/ip4/1")],
            },
            net_id::Peer {
                id: net_id::PeerId::from("a"),
                addrs: vec![net_id::PeerAddr::from("/ip4/2")],
            },
            net_id::Peer {
                id: net_id::PeerId::from("b"),
                addrs: Vec::new(),
            },
        ];
        add_candidates(&mut state, peers, discovery::UnixTime::from_secs(1));
        assert_eq!(state.candidates.len(), 1);
        assert!(state.candidates.contains_key(&net_id::PeerId::from("a")));
    }
}
