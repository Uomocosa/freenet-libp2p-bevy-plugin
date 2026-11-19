use std::collections::BTreeMap;

use crate::discovery;
use crate::p2p;
use crate::time;
use discovery::state_machine::{DiscoveryState, Output};

pub fn evict_held_rooms_over_history_limit(
    state: &mut DiscoveryState,
    bytes_by_prefix: &BTreeMap<p2p::HistoryKeyPrefix, p2p::HistoryByteCount>,
) -> Vec<Output> {
    let prefix_of = |room: &discovery::RoomName, created_at: time::UnixTime| {
        p2p::HistoryKeyPrefix::new(&state.game, room, created_at)
    };
    let bytes_at =
        |prefix: &p2p::HistoryKeyPrefix| bytes_by_prefix.get(prefix).copied().unwrap_or_default();
    let current_room_history_bytes = state
        .room
        .as_ref()
        .map_or_else(p2p::HistoryByteCount::default, |room| {
            bytes_at(&prefix_of(&room.name, room.created_at))
        });
    let history_bytes_by_held_room = state
        .held_rooms
        .iter()
        .map(|(room, held)| (room.clone(), bytes_at(&prefix_of(room, held.created_at))))
        .collect();
    let evicted = state.oldest_held_rooms_to_evict_until_history_limits_hold(
        &history_bytes_by_held_room,
        current_room_history_bytes,
    );
    let mut outputs = Vec::new();
    for room in evicted {
        let Some(held) = state.held_rooms.remove(&room) else {
            continue;
        };
        tracing::warn!(target: "discovery", room = %room.as_str(), "history over the size limit: evicted held room");
        outputs.push(Output::Libp2pCommand(
            p2p::Libp2pCommand::RemoveHistoryRecords {
                prefix: prefix_of(&room, held.created_at),
            },
        ));
        state.evicted_rooms.insert(room, held.created_at);
    }
    outputs
}
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::evict_held_rooms_over_history_limit;
    use crate::discovery;
    use crate::p2p;
    use crate::testing;
    use crate::time;
    use discovery::state_machine::Output;

    #[test]
    fn test_usage() {
        let room = discovery::RoomName::from("r");
        let mut state = testing::held_room_state("r", 10);
        assert_eq!(
            evict_held_rooms_over_history_limit(&mut state, &BTreeMap::new()),
            Vec::<Output>::new()
        );
        assert!(state.held_rooms.contains_key(&room));
    }

    #[test]
    fn evicted_history_publishes_departed_and_removes_records() {
        let room = discovery::RoomName::from("r");
        let mut state = testing::held_room_state("r", 10);
        let prefix = p2p::HistoryKeyPrefix::new(&state.game, &room, time::UnixTime::from_secs(1));
        let measured = BTreeMap::from([(prefix.clone(), p2p::HistoryByteCount(11))]);
        let outputs = evict_held_rooms_over_history_limit(&mut state, &measured);
        assert_eq!(
            outputs,
            vec![Output::Libp2pCommand(
                p2p::Libp2pCommand::RemoveHistoryRecords { prefix }
            )]
        );
        assert!(state.held_rooms.is_empty());
        let targets = state.heartbeat_targets();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].membership, discovery::RoomMembership::Departed);
    }
}
