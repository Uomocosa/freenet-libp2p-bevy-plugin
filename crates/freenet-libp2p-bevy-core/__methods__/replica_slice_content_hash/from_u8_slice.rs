use crate::replica_sync;

#[must_use]
pub fn from_u8_slice(bytes: &[u8]) -> replica_sync::ReplicaSliceContentHash {
    let hash = blake3::hash(bytes);
    let mut leading = [0u8; 8];
    for (slot, byte) in leading.iter_mut().zip(hash.as_bytes()) {
        *slot = *byte;
    }
    replica_sync::ReplicaSliceContentHash(u64::from_be_bytes(leading))
}

#[cfg(test)]
mod tests {
    use crate::replica_sync::ReplicaSliceContentHash;

    use super::from_u8_slice;

    #[test]
    fn test_usage() {
        let hash = from_u8_slice(b"tile");
        assert_eq!(hash, ReplicaSliceContentHash::from(b"tile".as_slice()));
        assert_ne!(hash, ReplicaSliceContentHash::from(b"tile!".as_slice()));
    }
}
