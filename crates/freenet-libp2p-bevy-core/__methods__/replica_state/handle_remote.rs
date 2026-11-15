use crate::identity;
use crate::replica_sync;

pub fn handle_remote<Op>(
    state: &mut replica_sync::state_machine::ReplicaState<Op>,
    zone: &identity::ZoneName,
    meta: replica_sync::OpMeta,
    op: Op,
) -> Vec<replica_sync::state_machine::Output<Op>> {
    if state.zone.as_ref() != Some(zone) {
        return Vec::new();
    }
    if state.phase == replica_sync::state_machine::Phase::Ready {
        vec![replica_sync::state_machine::Output::Apply { meta, op }]
    } else {
        state.buffered_ops.push((meta, op));
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::handle_remote;
    use crate::identity;
    use crate::replica_sync;
    use crate::replica_sync::state_machine::ReplicaState;

    #[test]
    fn test_usage() {
        let mut state = ReplicaState::<u8>::default();
        let outputs = handle_remote(
            &mut state,
            &identity::ZoneName::from("zone"),
            replica_sync::OpMeta {
                lamport: 1,
                author: 2,
            },
            7_u8,
        );
        assert_eq!(outputs, Vec::new());
        assert_eq!(state.buffered_ops, Vec::new());
    }
}
