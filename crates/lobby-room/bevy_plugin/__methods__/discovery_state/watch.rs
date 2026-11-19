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
    use super::watch;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![(
                "alpha",
                testing::room(discovery::RoomSize::DEFAULT, 10, Vec::new()),
            )]))
            .build();
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
            testing::held_room(&state, "alpha").expect("alpha is held"),
            testing::held_room_history(10, 20)
        );
    }

    #[test]
    fn watching_again_refreshes_only_left_at() {
        let mut state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![(
                "alpha",
                testing::room(discovery::RoomSize::DEFAULT, 10, Vec::new()),
            )]))
            .build();
        let room = discovery::RoomName::from("alpha");
        watch(&mut state, room.clone(), time::UnixTime::from_secs(20));
        watch(&mut state, room, time::UnixTime::from_secs(30));
        assert_eq!(
            testing::held_room(&state, "alpha").expect("alpha is held"),
            testing::held_room_history(10, 30)
        );
    }

    #[test]
    fn watching_while_in_a_room_does_nothing() {
        let mut state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![(
                "alpha",
                testing::room(discovery::RoomSize::DEFAULT, 10, Vec::new()),
            )]))
            .build();
        state.room = Some(testing::joined_room(
            "beta",
            0,
            discovery::JoinPhase::Ready,
            Vec::new(),
        ));
        watch(
            &mut state,
            discovery::RoomName::from("alpha"),
            time::UnixTime::from_secs(20),
        );
        assert!(state.held_rooms.is_empty());
    }

    #[test]
    fn watching_an_unlisted_room_does_nothing() {
        let mut state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![(
                "alpha",
                testing::room(discovery::RoomSize::DEFAULT, 10, Vec::new()),
            )]))
            .build();
        watch(
            &mut state,
            discovery::RoomName::from("missing"),
            time::UnixTime::from_secs(20),
        );
        assert!(state.held_rooms.is_empty());
    }

    #[test]
    fn watching_over_the_held_room_limit_is_evicted_oldest_watched_first() {
        let mut state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(testing::lobby_with(vec![
                (
                    "a",
                    testing::room(discovery::RoomSize::DEFAULT, 10, Vec::new()),
                ),
                (
                    "b",
                    testing::room(discovery::RoomSize::DEFAULT, 10, Vec::new()),
                ),
                (
                    "c",
                    testing::room(discovery::RoomSize::DEFAULT, 10, Vec::new()),
                ),
            ]))
            .build();
        state.history_size_limits.max_held_rooms = 2;
        for (room, watched_at) in [("a", 20), ("b", 30), ("c", 40)] {
            watch(
                &mut state,
                discovery::RoomName::from(room),
                time::UnixTime::from_secs(watched_at),
            );
        }
        let bytes = testing::history_bytes_by_room(&[("a", 1), ("b", 1), ("c", 1)]);
        let evicted = state.oldest_held_rooms_to_evict_until_history_limits_hold(
            &bytes,
            p2p::HistoryByteCount::default(),
        );
        assert_eq!(evicted, vec![discovery::RoomName::from("a")]);
    }
}
