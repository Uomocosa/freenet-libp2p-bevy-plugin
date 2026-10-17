use crate::room_sync;

#[must_use]
pub fn from_bytes(bytes: &[u8]) -> room_sync::ReplicaSliceContentHash {
    let hash = blake3::hash(bytes);
    let mut leading = [0u8; 8];
    for (slot, byte) in leading.iter_mut().zip(hash.as_bytes()) {
        *slot = *byte;
    }
    room_sync::ReplicaSliceContentHash(u64::from_be_bytes(leading))
}

#[cfg(test)]
mod tests {
    use crate::room_sync::ReplicaSliceContentHash;

    #[test]
    fn test_usage() {
        let hash = ReplicaSliceContentHash::from_bytes(b"tile");
        assert_eq!(hash, ReplicaSliceContentHash::from_bytes(b"tile"));
        assert_ne!(hash, ReplicaSliceContentHash::from_bytes(b"tile!"));
    }
}
