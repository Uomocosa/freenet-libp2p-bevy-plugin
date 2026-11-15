use crate::replica_sync;

pub fn reset_after_leaving<Op>(state: &mut replica_sync::bevy_systems::SyncState<Op>) {
    *state = replica_sync::bevy_systems::SyncState(replica_sync::state_machine::ReplicaState {
        slice_anti_entropy: replica_sync::state_machine::SliceAntiEntropyState {
            max_fetches_per_peer: state.slice_anti_entropy.max_fetches_per_peer,
            ..Default::default()
        },
        kademlia_history: replica_sync::state_machine::KademliaHistoryState {
            is_enabled: state.kademlia_history.is_enabled,
            max_fetch_retries: state.kademlia_history.max_fetch_retries,
            per_room_bytes: state.kademlia_history.per_room_bytes,
            record_ttl: state.kademlia_history.record_ttl,
            ..Default::default()
        },
        ..Default::default()
    });
}

#[cfg(test)]
mod tests {
    use super::reset_after_leaving;
    use crate::identity;
    use crate::replica_sync;

    #[test]
    fn test_usage() {
        let mut state = replica_sync::bevy_systems::SyncState::<u8>(
            replica_sync::state_machine::ReplicaState {
                room: Some(identity::RoomName::from("room")),
                kademlia_history: replica_sync::state_machine::KademliaHistoryState {
                    is_enabled: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );
        reset_after_leaving(&mut state);
        assert_eq!(state.room, None);
        assert!(state.kademlia_history.is_enabled);
    }
}
