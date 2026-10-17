use crate::discovery;
use discovery::state_machine::State;

pub fn seed_candidates(state: &mut State) {
    let Some(room) = &state.room else {
        return;
    };
    let Some(record) = state.lobby.get(&room.name) else {
        return;
    };
    let seeds: Vec<_> = record
        .members
        .iter()
        .filter(|(peer, heartbeat)| **peer != state.me.id && !heartbeat.addrs.is_empty())
        .map(|(peer, heartbeat)| (peer.clone(), heartbeat.clone()))
        .collect();
    for (peer, heartbeat) in seeds {
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
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use std::collections::BTreeMap;

    use super::seed_candidates;
    use crate::discovery;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let room = identity::RoomName::from("r");
        state.room = Some(discovery::JoinedRoom {
            name: room.clone(),
            members: discovery::Members::new(),
        });
        let heartbeat = discovery::Heartbeat {
            addrs: vec![identity::PeerAddr::from("/ip4/1")],
            updated_at: discovery::UnixTime::from_secs(5),
            display_name: None,
        };
        let members = BTreeMap::from([
            (identity::PeerId::from("me"), heartbeat.clone()),
            (identity::PeerId::from("a"), heartbeat),
        ]);
        state.lobby.insert(
            room,
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                members,
            },
        );
        seed_candidates(&mut state);
        let seeded: Vec<_> = state.candidates.keys().cloned().collect();
        assert_eq!(seeded, vec![identity::PeerId::from("a")]);
    }
}
