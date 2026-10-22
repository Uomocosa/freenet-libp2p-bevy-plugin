use std::collections::BTreeMap;

use crate::discovery;
use crate::identity;
use crate::p2p;
use discovery::state_machine::DiscoveryState;

#[must_use]
pub fn held_rooms_to_evict_until_history_fits(
    state: &DiscoveryState,
    held_bytes: &BTreeMap<identity::RoomName, p2p::HistoryByteCount>,
    current_room_bytes: p2p::HistoryByteCount,
    limit: p2p::HistoryByteCount,
) -> Vec<identity::RoomName> {
    let bytes_of = |room: &identity::RoomName| held_bytes.get(room).map_or(0, |bytes| **bytes);
    let total = state
        .held_rooms
        .keys()
        .map(bytes_of)
        .fold(*current_room_bytes, u64::saturating_add);
    let mut oldest_left_first: Vec<_> = state.held_rooms.iter().collect();
    oldest_left_first.sort_by_key(|(room, history)| (history.left_at, (*room).clone()));
    oldest_left_first
        .into_iter()
        .scan(total, |remaining, (room, _)| {
            if *remaining <= *limit {
                return None;
            }
            *remaining = remaining.saturating_sub(bytes_of(room));
            Some(room.clone())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::held_rooms_to_evict_until_history_fits;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
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
                    created_at: discovery::UnixTime::from_secs(0),
                    left_at: discovery::UnixTime::from_secs(*left_at),
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
        let evicted = held_rooms_to_evict_until_history_fits(
            &state_with_held(&[("a", 1)]),
            &bytes(&[("a", 10)]),
            p2p::HistoryByteCount(0),
            p2p::HistoryByteCount(5),
        );
        assert_eq!(evicted, vec![identity::RoomName::from("a")]);
    }

    #[test]
    fn held_room_left_first_is_evicted_first() {
        let evicted = held_rooms_to_evict_until_history_fits(
            &state_with_held(&[("new", 30), ("old", 10), ("mid", 20)]),
            &bytes(&[("new", 10), ("old", 10), ("mid", 10)]),
            p2p::HistoryByteCount(0),
            p2p::HistoryByteCount(15),
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
        let evicted = held_rooms_to_evict_until_history_fits(
            &state_with_held(&[("a", 1)]),
            &bytes(&[("a", 10)]),
            p2p::HistoryByteCount(5),
            p2p::HistoryByteCount(15),
        );
        assert_eq!(evicted, Vec::<identity::RoomName>::new());
    }

    #[test]
    fn current_room_alone_over_the_limit_evicts_every_held_room_but_itself() {
        let evicted = held_rooms_to_evict_until_history_fits(
            &state_with_held(&[("a", 1), ("b", 2)]),
            &bytes(&[("a", 1), ("b", 1)]),
            p2p::HistoryByteCount(100),
            p2p::HistoryByteCount(10),
        );
        assert_eq!(
            evicted,
            vec![identity::RoomName::from("a"), identity::RoomName::from("b")]
        );
    }
}
