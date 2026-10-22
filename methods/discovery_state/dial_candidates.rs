use crate::discovery;
use crate::identity;
use crate::p2p;
use discovery::state_machine::{DiscoveryState, Output};

#[must_use]
pub fn dial_candidates(state: &mut DiscoveryState, now: discovery::UnixTime) -> Vec<Output> {
    if state.room.is_none() {
        return Vec::new();
    }
    let redial_interval = discovery::state_machine::REDIAL_INTERVAL;
    let due: Vec<identity::Peer> = state
        .candidates
        .iter()
        .filter(|(peer, _)| !state.connected.contains(*peer))
        .filter(|(peer, _)| {
            state
                .last_dialed_at
                .get(*peer)
                .is_none_or(|last| now.elapsed_since(*last) >= redial_interval)
        })
        .map(|(peer, candidate)| identity::Peer {
            id: peer.clone(),
            addrs: candidate.addrs.clone(),
        })
        .collect();
    let mut outputs = Vec::new();
    for peer in due {
        tracing::debug!(target: "discovery", peer = %peer.id.as_str(), "discovery dial candidate");
        state.last_dialed_at.insert(peer.id.clone(), now);
        outputs.push(Output::NetCommand(p2p::NetCommand::Dial(peer)));
    }
    outputs
}

#[cfg(test)]
mod tests {
    use super::dial_candidates;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use discovery::state_machine::{DiscoveryState, Output};

    #[test]
    fn test_usage() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: discovery::UnixTime::from_secs(0),
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
        });
        let peers = vec![
            identity::Peer {
                id: identity::PeerId::from("a"),
                addrs: vec![identity::PeerAddr::from("/ip4/1")],
            },
            identity::Peer {
                id: identity::PeerId::from("b"),
                addrs: vec![identity::PeerAddr::from("/ip4/2")],
            },
        ];
        state.add_candidates(peers, discovery::UnixTime::from_secs(1));
        state.connected.insert(identity::PeerId::from("b"));
        let mut outputs = dial_candidates(&mut state, discovery::UnixTime::from_secs(10));
        outputs.extend(dial_candidates(
            &mut state,
            discovery::UnixTime::from_secs(10),
        ));
        assert!(matches!(
            outputs.as_slice(),
            [Output::NetCommand(p2p::NetCommand::Dial(peer))] if peer.id.as_str() == "a"
        ));
        assert_eq!(
            dial_candidates(&mut state, discovery::UnixTime::from_secs(12)).len(),
            1
        );
    }
}
