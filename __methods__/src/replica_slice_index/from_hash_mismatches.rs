use std::collections::BTreeMap;

use crate::room_sync;

#[must_use]
pub fn from_hash_mismatches(
    local: &room_sync::ReplicaSliceContentHashes,
    remote: &room_sync::ReplicaSliceContentHashes,
) -> Vec<room_sync::ReplicaSliceIndex> {
    let local_hash_by_index: BTreeMap<_, _> = local.iter().copied().collect();
    remote
        .iter()
        .filter(|(index, hash)| local_hash_by_index.get(index) != Some(hash))
        .map(|(index, _)| *index)
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::room_sync::{ReplicaSliceContentHash, ReplicaSliceIndex};

    fn hashes(entries: &[(u32, u64)]) -> Vec<(ReplicaSliceIndex, ReplicaSliceContentHash)> {
        entries
            .iter()
            .map(|(index, hash)| (ReplicaSliceIndex(*index), ReplicaSliceContentHash(*hash)))
            .collect()
    }

    #[test]
    fn test_usage() {
        let local = hashes(&[(0, 10), (1, 11), (3, 13)]);
        let remote = hashes(&[(0, 10), (1, 99), (2, 12)]);
        assert_eq!(
            ReplicaSliceIndex::from_hash_mismatches(&local, &remote),
            vec![ReplicaSliceIndex(1), ReplicaSliceIndex(2)]
        );
    }

    #[test]
    fn equal_hashes_have_no_mismatch() {
        let both = hashes(&[(0, 10), (5, 15)]);
        assert_eq!(
            ReplicaSliceIndex::from_hash_mismatches(&both, &both),
            Vec::new()
        );
    }

    #[test]
    fn a_slice_only_we_hold_is_not_fetched() {
        let local = hashes(&[(4, 14)]);
        assert_eq!(
            ReplicaSliceIndex::from_hash_mismatches(&local, &Vec::new()),
            Vec::new()
        );
    }
}
