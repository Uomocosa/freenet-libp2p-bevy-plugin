use std::collections::BTreeMap;

use crate::discovery;
use crate::identity;
use crate::p2p;
use discovery::state_machine::{DiscoveryState, Output};

pub fn drop_oldest_history(
    state: &mut DiscoveryState,
    bytes_by_prefix: &BTreeMap<p2p::HistoryKeyPrefix, p2p::HistoryByteCount>,
) -> Vec<Output> {
    let prefix_of = |room: &identity::RoomName, created_at: discovery::UnixTime| {
        p2p::HistoryKeyPrefix::new(&state.game, room, created_at)
    };
    let bytes_at =
        |prefix: &p2p::HistoryKeyPrefix| bytes_by_prefix.get(prefix).copied().unwrap_or_default();
    let current_room_bytes = state
        .room
        .as_ref()
        .map_or_else(p2p::HistoryByteCount::default, |room| {
            bytes_at(&prefix_of(&room.name, room.created_at))
        });
    let held_bytes = state
        .held_rooms
        .iter()
        .map(|(room, held)| (room.clone(), bytes_at(&prefix_of(room, held.created_at))))
        .collect();
    let evicted = state.held_rooms_to_evict_until_history_fits(
        &held_bytes,
        current_room_bytes,
        state.history_size_limits.per_process_bytes,
    );
    let mut outputs = Vec::new();
    for room in evicted {
        let Some(held) = state.held_rooms.remove(&room) else {
            continue;
        };
        tracing::warn!(target: "discovery", room = %room.as_str(), "history over the size limit: evicted held room");
        outputs.push(Output::NetCommand(p2p::NetCommand::RemoveHistoryRecords {
            prefix: prefix_of(&room, held.created_at),
        }));
        state.evicted_rooms.insert(room, held.created_at);
    }
    outputs
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::drop_oldest_history;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use discovery::state_machine::{DiscoveryState, Output};

    fn state_holding(room: &identity::RoomName) -> DiscoveryState {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        state.history_size_limits.per_process_bytes = p2p::HistoryByteCount(10);
        state.held_rooms.insert(
            room.clone(),
            discovery::state_machine::HeldRoomHistory {
                created_at: discovery::UnixTime::from_secs(1),
                left_at: discovery::UnixTime::from_secs(2),
            },
        );
        state
    }

    #[test]
    fn test_usage() {
        let room = identity::RoomName::from("r");
        let mut state = state_holding(&room);
        assert_eq!(
            drop_oldest_history(&mut state, &BTreeMap::new()),
            Vec::<Output>::new()
        );
        assert!(state.held_rooms.contains_key(&room));
    }

    #[test]
    fn evicted_history_publishes_quit_and_removes_records() {
        let room = identity::RoomName::from("r");
        let mut state = state_holding(&room);
        let prefix =
            p2p::HistoryKeyPrefix::new(&state.game, &room, discovery::UnixTime::from_secs(1));
        let measured = BTreeMap::from([(prefix.clone(), p2p::HistoryByteCount(11))]);
        let outputs = drop_oldest_history(&mut state, &measured);
        assert_eq!(
            outputs,
            vec![Output::NetCommand(p2p::NetCommand::RemoveHistoryRecords {
                prefix
            })]
        );
        assert!(state.held_rooms.is_empty());
        let targets = state.heartbeat_targets();
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].membership, discovery::RoomMembership::Quit);
    }
}
