use std::collections::BTreeMap;

use crate::discovery;
use crate::identity;
use crate::p2p;
use discovery::state_machine::DiscoveryState;

#[must_use]
pub fn oldest_held_rooms_to_evict_until_history_fits(
    state: &DiscoveryState,
    history_bytes_by_held_room: &BTreeMap<identity::RoomName, p2p::HistoryByteCount>,
    current_room_history_bytes: p2p::HistoryByteCount,
) -> Vec<identity::RoomName> {
    let history_bytes_of_room = |room: &identity::RoomName| {
        history_bytes_by_held_room
            .get(room)
            .map_or(0, |bytes| **bytes)
    };
    let limit = state.history_size_limits.per_process_bytes;
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
            total_history_bytes,
            |bytes_left_after_evictions, (room, _)| {
                if *bytes_left_after_evictions <= *limit {
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
                Some(room.clone())
            },
        )
        .collect()
}
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::oldest_held_rooms_to_evict_until_history_fits;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::time;
    use discovery::state_machine::DiscoveryState;

    fn state_with_held(rooms: &[(&str, u64)]) -> DiscoveryState {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        for (name, left_at) in rooms {
            state.held_rooms.insert(
                identity::RoomName::from(*name),
                discovery::state_machine::HeldRoomHistory {
                    created_at: time::UnixTime::EPOCH,
                    left_at: time::UnixTime::from_secs(*left_at),
                },
            );
        }
        state
    }

    fn bytes(rooms: &[(&str, u64)]) -> BTreeMap<identity::RoomName, p2p::HistoryByteCount> {
        rooms
            .iter()
            .map(|(name, bytes)| {
                (
                    identity::RoomName::from(*name),
                    p2p::HistoryByteCount(*bytes),
                )
            })
            .collect()
    }

    #[test]
    fn test_usage() {
        let mut state = state_with_held(&[("a", 1)]);
        state.history_size_limits.per_process_bytes = p2p::HistoryByteCount(5);
        let evicted = oldest_held_rooms_to_evict_until_history_fits(
            &state,
            &bytes(&[("a", 10)]),
            p2p::HistoryByteCount(0),
        );
        assert_eq!(evicted, vec![identity::RoomName::from("a")]);
    }

    #[test]
    fn held_room_left_first_is_evicted_first() {
        let mut state = state_with_held(&[("new", 30), ("old", 10), ("mid", 20)]);
        state.history_size_limits.per_process_bytes = p2p::HistoryByteCount(15);
        let evicted = oldest_held_rooms_to_evict_until_history_fits(
            &state,
            &bytes(&[("new", 10), ("old", 10), ("mid", 10)]),
            p2p::HistoryByteCount(0),
        );
        assert_eq!(
            evicted,
            vec![
                identity::RoomName::from("old"),
                identity::RoomName::from("mid")
            ]
        );
    }

    #[test]
    fn nothing_is_evicted_under_the_limit() {
        let mut state = state_with_held(&[("a", 1)]);
        state.history_size_limits.per_process_bytes = p2p::HistoryByteCount(15);
        let evicted = oldest_held_rooms_to_evict_until_history_fits(
            &state,
            &bytes(&[("a", 10)]),
            p2p::HistoryByteCount(5),
        );
        assert_eq!(evicted, Vec::<identity::RoomName>::new());
    }

    #[test]
    fn current_room_alone_over_the_limit_evicts_every_held_room_but_itself() {
        let mut state = state_with_held(&[("a", 1), ("b", 2)]);
        state.history_size_limits.per_process_bytes = p2p::HistoryByteCount(10);
        let evicted = oldest_held_rooms_to_evict_until_history_fits(
            &state,
            &bytes(&[("a", 1), ("b", 1)]),
            p2p::HistoryByteCount(100),
        );
        assert_eq!(
            evicted,
            vec![identity::RoomName::from("a"), identity::RoomName::from("b")]
        );
    }
}
