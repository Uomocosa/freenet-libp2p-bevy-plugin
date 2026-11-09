use crate::room_sync;

pub fn finish_member_fetch<Op>(
    state: &mut room_sync::state_machine::ReplicaState<Op>,
) -> Vec<room_sync::state_machine::Output<Op>> {
    if state.kademlia_history.is_enabled {
        return state.start_kademlia_history_fetch();
    }
    state.phase = room_sync::state_machine::Phase::Ready;
    let mut outputs = Vec::new();
    for (meta, op) in state.buffered_ops.drain(..) {
        outputs.push(room_sync::state_machine::Output::Apply { meta, op });
    }
    outputs
}

#[cfg(test)]
mod tests {
    use super::finish_member_fetch;
    use crate::room_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        assert_eq!(finish_member_fetch(&mut state), Vec::new());
    }
}
