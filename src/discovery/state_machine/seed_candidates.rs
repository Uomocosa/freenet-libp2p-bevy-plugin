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
        .filter(|(peer, presence)| **peer != state.me.id && !presence.addrs.is_empty())
        .map(|(peer, presence)| (peer.clone(), presence.clone()))
        .collect();
    for (peer, presence) in seeds {
        let newer = state
            .candidates
            .get(&peer)
            .is_none_or(|known| known.updated_at < presence.updated_at);
        if newer {
            state.candidates.insert(peer, presence);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::net_id;
    use std::collections::BTreeMap;

    use super::seed_candidates;
    use crate::discovery;
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
        let room = net_id::RoomName::from("r");
        state.room = Some(discovery::Room {
            name: room.clone(),
            members: discovery::Members::new(),
        });
        let presence = discovery::Presence {
            addrs: vec![net_id::PeerAddr::from("/ip4/1")],
            updated_at: discovery::UnixTime::from_secs(5),
        };
        let members = BTreeMap::from([
            (net_id::PeerId::from("me"), presence.clone()),
            (net_id::PeerId::from("a"), presence),
        ]);
        state.lobby.insert(
            room,
            discovery::RoomEntry {
                capacity: 8,
                members,
            },
        );
        seed_candidates(&mut state);
        let seeded: Vec<_> = state.candidates.keys().cloned().collect();
        assert_eq!(seeded, vec![net_id::PeerId::from("a")]);
    }
}
