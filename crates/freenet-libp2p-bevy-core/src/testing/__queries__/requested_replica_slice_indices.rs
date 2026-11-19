use crate::spectator;

#[must_use]
pub fn requested_replica_slice_indices(
    outputs: &[spectator::state_machine::SpectateOutput<u8, Vec<u8>>],
) -> Vec<u32> {
    outputs
        .iter()
        .filter_map(|output| match output {
            spectator::state_machine::SpectateOutput::RequestSlice { index, .. } => Some(**index),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::replica_sync;
    use crate::spectator;

    use super::requested_replica_slice_indices;

    #[test]
    fn test_usage() {
        let outputs = [
            spectator::state_machine::SpectateOutput::RequestSlice {
                from: identity::PeerId::from("holder"),
                index: replica_sync::ReplicaSliceIndex(1),
            },
            spectator::state_machine::SpectateOutput::Merge(Vec::new()),
        ];
        assert_eq!(requested_replica_slice_indices(&outputs), vec![1]);
    }
}
