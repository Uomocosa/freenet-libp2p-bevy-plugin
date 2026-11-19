use crate::replica_sync;

#[must_use]
pub fn hashes(entries: &[(u32, u64)]) -> replica_sync::ReplicaSliceContentHashes {
    entries
        .iter()
        .map(|(index, hash)| {
            (
                replica_sync::ReplicaSliceIndex(*index),
                replica_sync::ReplicaSliceContentHash(*hash),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::replica_sync;

    use super::hashes;

    #[test]
    fn test_usage() {
        assert_eq!(
            hashes(&[(0, 1), (2, 3)]),
            vec![
                (
                    replica_sync::ReplicaSliceIndex(0),
                    replica_sync::ReplicaSliceContentHash(1),
                ),
                (
                    replica_sync::ReplicaSliceIndex(2),
                    replica_sync::ReplicaSliceContentHash(3),
                ),
            ]
        );
    }
}
