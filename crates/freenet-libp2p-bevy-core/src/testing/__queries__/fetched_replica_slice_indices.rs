use crate::replica_sync;

#[must_use]
pub fn fetched_replica_slice_indices<Op>(
    outputs: &[replica_sync::state_machine::Output<Op>],
) -> Vec<u32> {
    outputs
        .iter()
        .filter_map(|output| match output {
            replica_sync::state_machine::Output::FetchReplicaSlice { index, .. } => Some(**index),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::replica_sync;

    use super::fetched_replica_slice_indices;

    #[test]
    fn test_usage() {
        let outputs = vec![
            replica_sync::state_machine::Output::<u8>::FetchReplicaSlice {
                peer: identity::PeerId::from("a"),
                zone: identity::ZoneName::from("r"),
                index: replica_sync::ReplicaSliceIndex(3),
            },
            replica_sync::state_machine::Output::Merge { bytes: vec![1] },
        ];
        assert_eq!(fetched_replica_slice_indices(&outputs), vec![3]);
    }
}
