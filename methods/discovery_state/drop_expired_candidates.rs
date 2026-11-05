use crate::discovery;
use discovery::state_machine::DiscoveryState;

pub fn drop_expired_candidates(state: &mut DiscoveryState, now: discovery::UnixTime) {
    let ttl = state.lease_limits.lease_ttl;
    state
        .candidates
        .retain(|_, candidate| now.elapsed_since(candidate.seen_at) <= ttl);
}

#[cfg(test)]
mod tests {
    use super::drop_expired_candidates;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::DiscoveryState;

    #[test]
    fn test_usage() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        for (peer, seen_at) in [("fresh", 90), ("stale", 0)] {
            state.candidates.insert(
                identity::PeerId::from(peer),
                discovery::state_machine::Candidate {
                    addrs: vec![identity::PeerAddr::from("/ip4/1")],
                    seen_at: discovery::UnixTime::from(seen_at),
                },
            );
        }
        drop_expired_candidates(&mut state, discovery::UnixTime::from(100));
        let left: Vec<_> = state.candidates.keys().cloned().collect();
        assert_eq!(left, vec![identity::PeerId::from("fresh")]);
    }
}
