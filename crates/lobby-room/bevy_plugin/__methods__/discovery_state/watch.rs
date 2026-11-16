use crate::discovery;
use crate::time;
use discovery::state_machine::{DiscoveryState, Output};

pub fn watch(
    state: &mut DiscoveryState,
    room: discovery::RoomName,
    now: time::UnixTime,
) -> Vec<Output> {
    if state.room.is_some() {
        return Vec::new();
    }
    let Some(lobby_room) = state.lobby.get(&room) else {
        return Vec::new();
    };
    let created_at = lobby_room.created_at;
    let held = state
        .held_rooms
        .entry(room)
        .or_insert(discovery::state_machine::HeldRoomHistory {
            created_at,
            left_at: now,
        });
    held.left_at = now;
    Vec::new()
}
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::watch;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::time;
    use discovery::state_machine::DiscoveryState;

    fn state_with_lobby(rooms: &[(&str, u64)]) -> DiscoveryState {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        for (name, created_at) in rooms {
            state.lobby.insert(
                discovery::RoomName::from(*name),
                discovery::Room {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at: time::UnixTime::from_secs(*created_at),
                    members: BTreeMap::new(),
                },
            );
        }
        state
    }

    fn held(state: &DiscoveryState, room: &str) -> discovery::state_machine::HeldRoomHistory {
        *state
            .held_rooms
            .get(&discovery::RoomName::from(room))
            .expect("room is held")
    }

    #[test]
    fn test_usage() {
        let mut state = state_with_lobby(&[("alpha", 10)]);
        assert_eq!(
            watch(
                &mut state,
                discovery::RoomName::from("alpha"),
                time::UnixTime::from_secs(20)
            ),
            Vec::new()
        );
        assert_eq!(state.held_rooms.len(), 1);
        assert_eq!(
            held(&state, "alpha"),
            discovery::state_machine::HeldRoomHistory {
                created_at: time::UnixTime::from_secs(10),
                left_at: time::UnixTime::from_secs(20),
            }
        );
    }

    #[test]
    fn watching_again_refreshes_only_left_at() {
        let mut state = state_with_lobby(&[("alpha", 10)]);
        let room = discovery::RoomName::from("alpha");
        watch(&mut state, room.clone(), time::UnixTime::from_secs(20));
        watch(&mut state, room, time::UnixTime::from_secs(30));
        assert_eq!(
            held(&state, "alpha"),
            discovery::state_machine::HeldRoomHistory {
                created_at: time::UnixTime::from_secs(10),
                left_at: time::UnixTime::from_secs(30),
            }
        );
    }

    #[test]
    fn watching_while_in_a_room_does_nothing() {
        let mut state = state_with_lobby(&[("alpha", 10)]);
        state.room = Some(discovery::JoinedRoom {
            name: discovery::RoomName::from("beta"),
            created_at: time::UnixTime::EPOCH,
            phase: discovery::JoinPhase::Ready,
            members: discovery::Members::new(),
            how_entered: discovery::HowRoomWasEntered::Created,
        });
        watch(
            &mut state,
            discovery::RoomName::from("alpha"),
            time::UnixTime::from_secs(20),
        );
        assert!(state.held_rooms.is_empty());
    }

    #[test]
    fn watching_an_unlisted_room_does_nothing() {
        let mut state = state_with_lobby(&[("alpha", 10)]);
        watch(
            &mut state,
            discovery::RoomName::from("missing"),
            time::UnixTime::from_secs(20),
        );
        assert!(state.held_rooms.is_empty());
    }

    #[test]
    fn watching_over_the_held_room_limit_is_evicted_oldest_watched_first() {
        let mut state = state_with_lobby(&[("a", 10), ("b", 10), ("c", 10)]);
        state.history_size_limits.max_held_rooms = 2;
        for (room, watched_at) in [("a", 20), ("b", 30), ("c", 40)] {
            watch(
                &mut state,
                discovery::RoomName::from(room),
                time::UnixTime::from_secs(watched_at),
            );
        }
        let bytes = BTreeMap::from([
            (discovery::RoomName::from("a"), p2p::HistoryByteCount(1)),
            (discovery::RoomName::from("b"), p2p::HistoryByteCount(1)),
            (discovery::RoomName::from("c"), p2p::HistoryByteCount(1)),
        ]);
        let evicted = state.oldest_held_rooms_to_evict_until_history_limits_hold(
            &bytes,
            p2p::HistoryByteCount::default(),
        );
        assert_eq!(evicted, vec![discovery::RoomName::from("a")]);
    }
}
