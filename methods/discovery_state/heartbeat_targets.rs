use crate::discovery;
use discovery::state_machine::{DiscoveryState, HeartbeatTarget};

#[must_use]
pub fn heartbeat_targets(state: &DiscoveryState) -> Vec<HeartbeatTarget> {
    let occupants = state.room.iter().map(|room| {
        (
            room.name.clone(),
            room.created_at,
            discovery::RoomMembership::Occupant,
        )
    });
    let history_holders = state.held_rooms.iter().map(|(name, held)| {
        (
            name.clone(),
            held.created_at,
            discovery::RoomMembership::HistoryHolder,
        )
    });
    let departed = state.evicted_rooms.iter().map(|(name, created_at)| {
        (
            name.clone(),
            *created_at,
            discovery::RoomMembership::Departed,
        )
    });
    occupants
        .chain(history_holders)
        .chain(departed)
        .map(|(room, created_at, membership)| HeartbeatTarget {
            room,
            created_at,
            addrs: state.me.addrs.clone(),
            display_name: state.display_name.clone(),
            membership: if state.is_quitting {
                discovery::RoomMembership::Departed
            } else {
                membership
            },
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::heartbeat_targets;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::DiscoveryState;

    #[test]
    fn test_usage() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/9")],
        });
        assert_eq!(heartbeat_targets(&state), Vec::new());
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            created_at: discovery::UnixTime::from(0),
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
            entry: discovery::RoomEntry::Created,
        });
        state.held_rooms.insert(
            identity::RoomName::from("old"),
            discovery::state_machine::HeldRoomHistory {
                created_at: discovery::UnixTime::from(0),
                left_at: discovery::UnixTime::from(5),
            },
        );
        state.evicted_rooms.insert(
            identity::RoomName::from("gone"),
            discovery::UnixTime::from(0),
        );
        let presences: Vec<_> = heartbeat_targets(&state)
            .into_iter()
            .map(|target| (target.room, target.membership))
            .collect();
        assert_eq!(
            presences,
            vec![
                (
                    identity::RoomName::from("r"),
                    discovery::RoomMembership::Occupant
                ),
                (
                    identity::RoomName::from("old"),
                    discovery::RoomMembership::HistoryHolder
                ),
                (
                    identity::RoomName::from("gone"),
                    discovery::RoomMembership::Departed
                ),
            ]
        );
        state.is_quitting = true;
        assert!(
            heartbeat_targets(&state)
                .iter()
                .all(|target| target.membership == discovery::RoomMembership::Departed)
        );
    }
}
