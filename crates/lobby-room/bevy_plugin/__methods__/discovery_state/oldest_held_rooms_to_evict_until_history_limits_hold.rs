use std::collections::BTreeMap;

use crate::discovery;
use crate::p2p;
use discovery::state_machine::DiscoveryState;

#[must_use]
pub fn oldest_held_rooms_to_evict_until_history_limits_hold(
    state: &DiscoveryState,
    history_bytes_by_held_room: &BTreeMap<discovery::RoomName, p2p::HistoryByteCount>,
    current_room_history_bytes: p2p::HistoryByteCount,
) -> Vec<discovery::RoomName> {
    let history_bytes_of_room = |room: &discovery::RoomName| {
        history_bytes_by_held_room
            .get(room)
            .map_or(0, |bytes| **bytes)
    };
    let limit = state.history_size_limits.per_process_bytes;
    let max_held_rooms = state.history_size_limits.max_held_rooms;
    let total_history_bytes = state
        .held_rooms
        .keys()
        .map(history_bytes_of_room)
        .fold(*current_room_history_bytes, u64::saturating_add);
    tracing::debug!(
        target: "discovery",
        total_history_bytes,
        limit_bytes = *limit,
        "history size against the per-process limit"
    );
    let mut oldest_left_first: Vec<_> = state.held_rooms.iter().collect();
    oldest_left_first.sort_by_key(|(room, history)| (history.left_at, (*room).clone()));
    oldest_left_first
        .into_iter()
        .scan(
            (total_history_bytes, state.held_rooms.len()),
            |(bytes_left_after_evictions, rooms_left_after_evictions), (room, _)| {
                let is_over_bytes = *bytes_left_after_evictions > *limit;
                let is_over_count = *rooms_left_after_evictions > max_held_rooms;
                if !is_over_bytes && !is_over_count {
                    return None;
                }
                let room_history_bytes = history_bytes_of_room(room);
                tracing::debug!(
                    target: "discovery",
                    room = %room.as_str(),
                    room_history_bytes,
                    "evicting held room over the history limit"
                );
                *bytes_left_after_evictions =
                    bytes_left_after_evictions.saturating_sub(room_history_bytes);
                *rooms_left_after_evictions = rooms_left_after_evictions.saturating_sub(1);
                Some(room.clone())
            },
        )
        .collect()
}
#[cfg(test)]
mod tests {
    use super::oldest_held_rooms_to_evict_until_history_limits_hold;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_held_rooms(testing::held_rooms_by_room(&[("a", 1)]))
            .build();
        state.history_size_limits.per_process_bytes = p2p::HistoryByteCount(5);
        let evicted = oldest_held_rooms_to_evict_until_history_limits_hold(
            &state,
            &testing::history_bytes_by_room(&[("a", 10)]),
            p2p::HistoryByteCount(0),
        );
        assert_eq!(evicted, vec![discovery::RoomName::from("a")]);
    }

    #[test]
    fn held_room_left_first_is_evicted_first() {
        let mut state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_held_rooms(testing::held_rooms_by_room(&[
                ("new", 30),
                ("old", 10),
                ("mid", 20),
            ]))
            .build();
        state.history_size_limits.per_process_bytes = p2p::HistoryByteCount(15);
        let evicted = oldest_held_rooms_to_evict_until_history_limits_hold(
            &state,
            &testing::history_bytes_by_room(&[("new", 10), ("old", 10), ("mid", 10)]),
            p2p::HistoryByteCount(0),
        );
        assert_eq!(
            evicted,
            vec![
                discovery::RoomName::from("old"),
                discovery::RoomName::from("mid")
            ]
        );
    }

    #[test]
    fn leaving_a_third_room_evicts_the_one_left_first_when_two_are_allowed() {
        let mut state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_held_rooms(testing::held_rooms_by_room(&[
                ("c", 30),
                ("a", 10),
                ("b", 20),
            ]))
            .build();
        state.history_size_limits.max_held_rooms = 2;
        let evicted = oldest_held_rooms_to_evict_until_history_limits_hold(
            &state,
            &testing::history_bytes_by_room(&[("a", 1), ("b", 1), ("c", 1)]),
            p2p::HistoryByteCount(0),
        );
        assert_eq!(evicted, vec![discovery::RoomName::from("a")]);
    }

    #[test]
    fn rejoined_room_is_not_counted_twice() {
        let mut state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_held_rooms(testing::held_rooms_by_room(&[("a", 10), ("b", 20)]))
            .build();
        state.history_size_limits.max_held_rooms = 2;
        let evicted = oldest_held_rooms_to_evict_until_history_limits_hold(
            &state,
            &testing::history_bytes_by_room(&[("a", 1), ("b", 1)]),
            p2p::HistoryByteCount(1),
        );
        assert_eq!(evicted, Vec::<discovery::RoomName>::new());
    }

    #[test]
    fn nothing_is_evicted_under_the_limit() {
        let mut state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_held_rooms(testing::held_rooms_by_room(&[("a", 1)]))
            .build();
        state.history_size_limits.per_process_bytes = p2p::HistoryByteCount(15);
        let evicted = oldest_held_rooms_to_evict_until_history_limits_hold(
            &state,
            &testing::history_bytes_by_room(&[("a", 10)]),
            p2p::HistoryByteCount(5),
        );
        assert_eq!(evicted, Vec::<discovery::RoomName>::new());
    }

    #[test]
    fn current_room_alone_over_the_limit_evicts_every_held_room_but_itself() {
        let mut state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_held_rooms(testing::held_rooms_by_room(&[("a", 1), ("b", 2)]))
            .build();
        state.history_size_limits.per_process_bytes = p2p::HistoryByteCount(10);
        let evicted = oldest_held_rooms_to_evict_until_history_limits_hold(
            &state,
            &testing::history_bytes_by_room(&[("a", 1), ("b", 1)]),
            p2p::HistoryByteCount(100),
        );
        assert_eq!(
            evicted,
            vec![
                discovery::RoomName::from("a"),
                discovery::RoomName::from("b")
            ]
        );
    }
}
