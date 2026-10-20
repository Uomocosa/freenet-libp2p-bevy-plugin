use crate::discovery;
use discovery::state_machine::{HeartbeatTarget, State};

#[must_use]
pub fn heartbeat_targets(state: &State) -> Vec<HeartbeatTarget> {
    let in_room = state.room.iter().map(|room| {
        (
            room.name.clone(),
            room.created_at,
            discovery::Presence::InRoom,
        )
    });
    let holding = state.held_rooms.iter().map(|(name, held)| {
        (
            name.clone(),
            held.created_at,
            discovery::Presence::HoldingHistory,
        )
    });
    let evicted = state
        .evicted_rooms
        .iter()
        .map(|(name, created_at)| (name.clone(), *created_at, discovery::Presence::Quit));
    in_room
        .chain(holding)
        .chain(evicted)
        .map(|(room, created_at, presence)| HeartbeatTarget {
            room,
            created_at,
            addrs: state.me.addrs.clone(),
            display_name: state.display_name.clone(),
            presence: if state.is_quitting {
                discovery::Presence::Quit
            } else {
                presence
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::heartbeat_targets;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/9")],
        });
        assert_eq!(heartbeat_targets(&state), Vec::new());
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: discovery::UnixTime::from_secs(0),
            phase: discovery::RoomPhase::Ready,
            members: discovery::Members::new(),
        });
        state.held_rooms.insert(
            identity::RoomName::from("old"),
            discovery::state_machine::HeldRoomHistory {
                created_at: discovery::UnixTime::from_secs(0),
                left_at: discovery::UnixTime::from_secs(5),
            },
        );
        state.evicted_rooms.insert(
            identity::RoomName::from("gone"),
            discovery::UnixTime::from_secs(0),
        );
        let presences: Vec<_> = heartbeat_targets(&state)
            .into_iter()
            .map(|target| (target.room, target.presence))
            .collect();
        assert_eq!(
            presences,
            vec![
                (identity::RoomName::from("r"), discovery::Presence::InRoom),
                (
                    identity::RoomName::from("old"),
                    discovery::Presence::HoldingHistory
                ),
                (identity::RoomName::from("gone"), discovery::Presence::Quit),
            ]
        );
        state.is_quitting = true;
        assert!(
            heartbeat_targets(&state)
                .iter()
                .all(|target| target.presence == discovery::Presence::Quit)
        );
    }
}
