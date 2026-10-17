use crate::discovery;
use crate::identity;
use discovery::state_machine::{Handshake, Output, State};

pub fn handle_handshake(
    state: &mut State,
    from: &identity::PeerId,
    data: &[u8],
    now: discovery::UnixTime,
) -> Vec<Output> {
    let Ok(handshake) = bincode::deserialize::<Handshake>(data) else {
        return Vec::new();
    };
    let my_room = state.room.as_ref().map(|room| room.name.clone());
    if handshake.room.is_none() || handshake.room != my_room {
        return drop_member(state, from);
    }
    let mut peers = handshake.peers;
    peers.push(identity::Peer {
        id: from.clone(),
        addrs: handshake.addrs.clone(),
    });
    discovery::state_machine::add_candidates(state, peers, now);
    if !state.connected.contains(from) {
        return Vec::new();
    }
    let Some(room) = state.room.as_mut() else {
        return Vec::new();
    };
    let member = discovery::Member {
        heartbeat: discovery::Heartbeat {
            addrs: handshake.addrs,
            updated_at: now,
            display_name: Some(handshake.display_name),
        },
        status: discovery::LinkStatus::Connected,
    };
    let added = room.members.insert(from.clone(), member).is_none();
    if !added {
        return Vec::new();
    }
    tracing::info!(
        target: "discovery",
        peer = %from.as_str(),
        room = %room.name.as_str(),
        members = room.members.len(),
        "discovery member joined"
    );
    let mut outputs = vec![Output::Notify(discovery::Event::MembersChanged)];
    outputs.extend(discovery::state_machine::send_handshake(state, from));
    outputs
}

// needed helper: removes a peer that left our room or switched to another one
fn drop_member(state: &mut State, peer: &identity::PeerId) -> Vec<Output> {
    let removed = state
        .room
        .as_mut()
        .and_then(|room| room.members.remove(peer))
        .is_some();
    if !removed {
        return Vec::new();
    }
    tracing::info!(target: "discovery", peer = %peer.as_str(), "discovery member left");
    vec![Output::Notify(discovery::Event::MembersChanged)]
}

#[cfg(test)]
mod tests {
    use super::handle_handshake;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use discovery::state_machine::{Handshake, Output, State};

    fn session_in_room() -> State {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: discovery::UnixTime::from_secs(0),
            phase: discovery::RoomPhase::Ready,
            members: discovery::Members::new(),
        });
        state
    }

    #[test]
    fn test_usage() {
        let mut state = session_in_room();
        let peer = identity::PeerId::from("a");
        state.connected.insert(peer.clone());
        let handshake = Handshake {
            room: Some(identity::RoomName::from("r")),
            addrs: vec![identity::PeerAddr::from("/ip4/1")],
            peers: vec![identity::Peer {
                id: identity::PeerId::from("b"),
                addrs: vec![identity::PeerAddr::from("/ip4/2")],
            }],
            display_name: identity::Username::from("a"),
        };
        let data = bincode::serialize(&handshake).unwrap_or_default();
        let outputs = handle_handshake(&mut state, &peer, &data, discovery::UnixTime::from_secs(1));
        assert_eq!(state.room.as_ref().map_or(0, |room| room.members.len()), 1);
        assert_eq!(state.candidates.len(), 2);
        assert!(matches!(
            outputs.as_slice(),
            [
                Output::Notify(discovery::Event::MembersChanged),
                Output::NetCommand(p2p::NetCommand::PeerExchange { .. }),
            ]
        ));
        let left = bincode::serialize(&Handshake::default()).unwrap_or_default();
        let _ = handle_handshake(&mut state, &peer, &left, discovery::UnixTime::from_secs(2));
        assert_eq!(state.room.as_ref().map_or(0, |room| room.members.len()), 0);
    }

    #[test]
    fn test_connected_before_known_becomes_member() {
        let mut state = session_in_room();
        let joiner = identity::PeerId::from("joiner");
        state.connected.insert(joiner.clone());
        let handshake = Handshake {
            room: Some(identity::RoomName::from("r")),
            addrs: vec![identity::PeerAddr::from("/ip4/1")],
            peers: Vec::new(),
            display_name: identity::Username::from("a"),
        };
        let data = bincode::serialize(&handshake).unwrap_or_default();
        let _ = handle_handshake(
            &mut state,
            &joiner,
            &data,
            discovery::UnixTime::from_secs(1),
        );
        let status = state
            .room
            .as_ref()
            .and_then(|room| room.members.get(&joiner))
            .map(|member| member.status);
        assert_eq!(status, Some(discovery::LinkStatus::Connected));
    }
}
