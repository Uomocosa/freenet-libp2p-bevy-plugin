use crate::identity;
use crate::room_sync;

pub fn handle_replica_slice_request_failed<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    from: &identity::PeerId,
    index: room_sync::ReplicaSliceIndex,
) -> Vec<room_sync::state_machine::Output<Op>> {
    let should_remove_peer = state
        .slice_anti_entropy
        .fetches_awaiting_reply
        .get_mut(from)
        .is_some_and(|awaiting| {
            awaiting.remove(&index);
            awaiting.is_empty()
        });
    if should_remove_peer {
        state.slice_anti_entropy.fetches_awaiting_reply.remove(from);
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::handle_replica_slice_request_failed;
    use crate::identity;
    use crate::room_sync;
    use crate::room_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let peer = identity::PeerId::from("peer");
        let index = room_sync::ReplicaSliceIndex(0);
        state
            .slice_anti_entropy
            .fetches_awaiting_reply
            .entry(peer.clone())
            .or_default()
            .insert(index, room_sync::ElapsedSinceAppStart::default());
        assert_eq!(
            handle_replica_slice_request_failed(&mut state, &peer, index),
            Vec::new()
        );
        assert!(
            !state
                .slice_anti_entropy
                .fetches_awaiting_reply
                .contains_key(&peer)
        );
    }
}
