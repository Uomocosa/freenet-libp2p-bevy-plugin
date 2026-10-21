use crate::discovery;
use crate::identity;
use crate::p2p;
use discovery::state_machine::{Output, State};

#[must_use]
pub fn collect_room_peers_from_lobby(state: &mut State) -> Vec<Output> {
    let Some(room) = &state.room else {
        return Vec::new();
    };
    let Some(record) = state.lobby.get(&room.name) else {
        return Vec::new();
    };
    let seeds: Vec<_> = record
        .members
        .iter()
        .filter(|(peer, heartbeat)| {
            **peer != state.me.id
                && !heartbeat.addrs.is_empty()
                && heartbeat.presence != discovery::Presence::Quit
        })
        .map(|(peer, heartbeat)| (peer.clone(), heartbeat.clone()))
        .collect();
    let mut outputs = Vec::new();
    for (peer, heartbeat) in seeds {
        outputs.push(Output::NetCommand(p2p::NetCommand::AddKadPeer(
            identity::Peer {
                id: peer.clone(),
                addrs: heartbeat.addrs.clone(),
            },
        )));
        if heartbeat.presence != discovery::Presence::InRoom {
            continue;
        }
        let newer = state
            .candidates
            .get(&peer)
            .is_none_or(|known| known.seen_at < heartbeat.updated_at);
        if newer {
            state.candidates.insert(
                peer,
                discovery::state_machine::Candidate {
                    addrs: heartbeat.addrs,
                    seen_at: heartbeat.updated_at,
                },
            );
        }
    }
    outputs
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::collect_room_peers_from_lobby;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use discovery::state_machine::{Output, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let room = identity::RoomName::from("r");
        state.room = Some(discovery::JoinedRoom {
            name: room.clone(),
            created_at: discovery::UnixTime::from_secs(0),
            phase: discovery::RoomPhase::Ready,
            members: discovery::Members::new(),
        });
        let in_room = discovery::Heartbeat {
            addrs: vec![identity::PeerAddr::from("/ip4/1")],
            updated_at: discovery::UnixTime::from_secs(5),
            display_name: None,
            presence: discovery::Presence::InRoom,
        };
        let holder = discovery::Heartbeat {
            addrs: vec![identity::PeerAddr::from("/ip4/2")],
            updated_at: discovery::UnixTime::from_secs(5),
            display_name: None,
            presence: discovery::Presence::HoldingHistory,
        };
        let members = BTreeMap::from([
            (identity::PeerId::from("me"), in_room.clone()),
            (identity::PeerId::from("a"), in_room),
            (identity::PeerId::from("holder"), holder),
        ]);
        state.lobby.insert(
            room,
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from_secs(0),
                members,
            },
        );
        let outputs = collect_room_peers_from_lobby(&mut state);
        let seeded: Vec<_> = state.candidates.keys().cloned().collect();
        assert_eq!(seeded, vec![identity::PeerId::from("a")]);
        assert_eq!(
            outputs,
            vec![
                Output::NetCommand(p2p::NetCommand::AddKadPeer(identity::Peer {
                    id: identity::PeerId::from("a"),
                    addrs: vec![identity::PeerAddr::from("/ip4/1")],
                })),
                Output::NetCommand(p2p::NetCommand::AddKadPeer(identity::Peer {
                    id: identity::PeerId::from("holder"),
                    addrs: vec![identity::PeerAddr::from("/ip4/2")],
                })),
            ]
        );
    }
}
