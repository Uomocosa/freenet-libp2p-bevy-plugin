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
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let addr = "/ip4/127.0.0.1/tcp/4001";
        let mut state = discovery::state_machine::DiscoveryState::new(testing::peer("me", &[addr]));
        add_candidates(
            &mut state,
            vec![
                testing::peer("me", &[addr]),
                testing::peer("ghost", &[]),
                testing::peer("alice", &[addr]),
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
