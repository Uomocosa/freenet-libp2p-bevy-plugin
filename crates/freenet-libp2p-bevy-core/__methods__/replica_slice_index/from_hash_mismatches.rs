use std::collections::BTreeMap;

use crate::replica_sync;

#[must_use]
pub fn from_hash_mismatches(
    local: &replica_sync::ReplicaSliceContentHashes,
    remote: &replica_sync::ReplicaSliceContentHashes,
) -> Vec<replica_sync::ReplicaSliceIndex> {
    let local_hash_by_index: BTreeMap<_, _> = local.iter().copied().collect();
    remote
        .iter()
        .filter(|(index, hash)| local_hash_by_index.get(index) != Some(hash))
        .map(|(index, _)| *index)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::replica_sync::ReplicaSliceIndex;
    use crate::testing;

    #[test]
    fn test_usage() {
        let local = testing::hashes(&[(0, 10), (1, 11), (3, 13)]);
        let remote = testing::hashes(&[(0, 10), (1, 99), (2, 12)]);
        assert_eq!(
            ReplicaSliceIndex::from_hash_mismatches(&local, &remote),
            vec![ReplicaSliceIndex(1), ReplicaSliceIndex(2)]
        );
    }

    #[test]
    fn equal_hashes_have_no_mismatch() {
        let both = testing::hashes(&[(0, 10), (5, 15)]);
        assert_eq!(
            ReplicaSliceIndex::from_hash_mismatches(&both, &both),
            Vec::new()
        );
    }

    #[test]
    fn a_slice_only_we_hold_is_not_fetched() {
        let local = testing::hashes(&[(4, 14)]);
        assert_eq!(
            ReplicaSliceIndex::from_hash_mismatches(&local, &Vec::new()),
            Vec::new()
        );
    }
}
