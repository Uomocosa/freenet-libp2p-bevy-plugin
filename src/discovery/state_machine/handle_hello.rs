use crate::discovery;
use crate::net_id;
use discovery::state_machine::{Hello, Output, State};

pub fn handle_hello(
    state: &mut State,
    from: &net_id::PeerId,
    data: &[u8],
    now: discovery::UnixTime,
) -> Vec<Output> {
    let Ok(hello) = bincode::deserialize::<Hello>(data) else {
        return Vec::new();
    };
    let my_room = state.room.as_ref().map(|room| room.name.clone());
    if hello.room.is_none() || hello.room != my_room {
        return drop_member(state, from);
    }
    let mut peers = hello.peers;
    peers.push(net_id::Peer {
        id: from.clone(),
        addrs: hello.addrs.clone(),
    });
    discovery::state_machine::add_candidates(state, peers, now);
    if !state.connected.contains(from) {
        return Vec::new();
    }
    let Some(room) = state.room.as_mut() else {
        return Vec::new();
    };
    let member = discovery::Member {
        presence: discovery::Presence {
            addrs: hello.addrs,
            updated_at: now,
        },
        status: discovery::LinkStatus::Connected,
    };
    let added = room.members.insert(from.clone(), member).is_none();
    if !added {
        return Vec::new();
    }
    tracing::info!(
        target: "room_lobby",
        peer = %from.as_str(),
        room = %room.name.as_str(),
        members = room.members.len(),
        "discovery member joined"
    );
    let mut outputs = vec![Output::Notify(discovery::Event::MembersChanged)];
    outputs.extend(discovery::state_machine::send_hello(state, from));
    outputs
}

// needed helper: removes a peer that left our room or switched to another one
fn drop_member(state: &mut State, peer: &net_id::PeerId) -> Vec<Output> {
    let removed = state
        .room
        .as_mut()
        .and_then(|room| room.members.remove(peer))
        .is_some();
    if !removed {
        return Vec::new();
    }
    tracing::info!(target: "room_lobby", peer = %peer.as_str(), "discovery member left");
    vec![Output::Notify(discovery::Event::MembersChanged)]
}

#[cfg(test)]
mod tests {
    use super::handle_hello;
    use crate::discovery;
    use crate::net_id;
    use crate::p2p;
    use discovery::state_machine::{Hello, Output, State};

    fn session_in_room() -> State {
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
        state
    }

    #[test]
    fn test_usage() {
        let mut state = session_in_room();
        let peer = net_id::PeerId::from("a");
        state.connected.insert(peer.clone());
        let hello = Hello {
            room: Some(net_id::RoomName::from("r")),
            addrs: vec![net_id::PeerAddr::from("/ip4/1")],
            peers: vec![net_id::Peer {
                id: net_id::PeerId::from("b"),
                addrs: vec![net_id::PeerAddr::from("/ip4/2")],
            }],
        };
        let data = bincode::serialize(&hello).unwrap_or_default();
        let outputs = handle_hello(&mut state, &peer, &data, discovery::UnixTime::from_secs(1));
        assert_eq!(state.room.as_ref().map_or(0, |room| room.members.len()), 1);
        assert_eq!(state.candidates.len(), 2);
        assert!(matches!(
            outputs.as_slice(),
            [
                Output::Notify(discovery::Event::MembersChanged),
                Output::NetCommand(p2p::NetCommand::Exchange { .. }),
            ]
        ));
        let left = bincode::serialize(&Hello::default()).unwrap_or_default();
        let _ = handle_hello(&mut state, &peer, &left, discovery::UnixTime::from_secs(2));
        assert_eq!(state.room.as_ref().map_or(0, |room| room.members.len()), 0);
    }

    #[test]
    fn test_connected_before_known_becomes_member() {
        let mut state = session_in_room();
        let joiner = net_id::PeerId::from("joiner");
        state.connected.insert(joiner.clone());
        let hello = Hello {
            room: Some(net_id::RoomName::from("r")),
            addrs: vec![net_id::PeerAddr::from("/ip4/1")],
            peers: Vec::new(),
        };
        let data = bincode::serialize(&hello).unwrap_or_default();
        let _ = handle_hello(
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
