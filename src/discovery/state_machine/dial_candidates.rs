use crate::discovery;
use crate::net_id;
use crate::p2p;
use discovery::state_machine::{Output, State};

#[must_use]
pub fn dial_candidates(state: &mut State, now: discovery::UnixTime) -> Vec<Output> {
    if state.room.is_none() {
        return Vec::new();
    }
    let redial = state.timing.redial;
    let due: Vec<net_id::Peer> = state
        .candidates
        .iter()
        .filter(|(peer, _)| !state.connected.contains(*peer))
        .filter(|(peer, _)| {
            state
                .last_dial
                .get(*peer)
                .is_none_or(|last| now.since(*last) >= redial)
        })
        .map(|(peer, presence)| net_id::Peer {
            id: peer.clone(),
            addrs: presence.addrs.clone(),
        })
        .collect();
    let mut outputs = Vec::new();
    for peer in due {
        tracing::debug!(target: "room_lobby", peer = %peer.id.as_str(), "discovery dial candidate");
        state.last_dial.insert(peer.id.clone(), now);
        outputs.push(Output::NetCommand(p2p::NetCommand::Dial(peer)));
    }
    outputs
}

#[cfg(test)]
mod tests {
    use super::dial_candidates;
    use crate::discovery;
    use crate::net_id;
    use crate::p2p;
    use discovery::state_machine::{Output, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: Vec::new(),
            },
            discovery::Timing::default(),
        );
        state.room = Some(discovery::Room {
            name: net_id::RoomName::from("r"),
            members: discovery::Members::new(),
        });
        let peers = vec![
            net_id::Peer {
                id: net_id::PeerId::from("a"),
                addrs: vec![net_id::PeerAddr::from("/ip4/1")],
            },
            net_id::Peer {
                id: net_id::PeerId::from("b"),
                addrs: vec![net_id::PeerAddr::from("/ip4/2")],
            },
        ];
        discovery::state_machine::add_candidates(
            &mut state,
            peers,
            discovery::UnixTime::from_secs(1),
        );
        state.connected.insert(net_id::PeerId::from("b"));
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
