use std::collections::BTreeMap;

use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::testing;

#[must_use]
pub fn held_room_state(
    room: &str,
    per_process_bytes: u64,
) -> discovery::state_machine::DiscoveryState {
    let mut state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
        .with_held_rooms(BTreeMap::from([(
            discovery::RoomName::from(room),
            testing::held_room_history(1, 2),
        )]))
        .build();
    state.history_size_limits.per_process_bytes = p2p::HistoryByteCount(per_process_bytes);
    state
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::p2p;

    use super::held_room_state;

    #[test]
    fn test_usage() {
        let state = held_room_state("r", 10);
        assert!(
            state
                .held_rooms
                .contains_key(&discovery::RoomName::from("r"))
        );
        assert_eq!(
            state.history_size_limits.per_process_bytes,
            p2p::HistoryByteCount(10)
        );
    }
}
