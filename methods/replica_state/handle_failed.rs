use crate::identity;
use crate::room_sync;

pub fn handle_failed<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
    from: &identity::PeerId,
) -> Vec<room_sync::state_machine::Output<Op>> {
    state.slice_anti_entropy.fetches_awaiting_reply.remove(from);
    state.retry_fetch()
}

#[cfg(test)]
mod tests {
    use super::handle_failed;
    use crate::identity;
    use crate::room_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        assert_eq!(
            handle_failed(&mut state, &identity::PeerId::from("peer")),
            Vec::new()
        );
    }
}
