#[must_use]
pub fn join_parts(parts: &[Vec<u8>]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for part in parts {
        bytes.extend_from_slice(part);
    }
    bytes
}

#[cfg(test)]
mod tests {
    use crate::room_sync;

    #[test]
    fn test_usage() {
        assert_eq!(room_sync::join_parts(&[vec![1, 2], vec![3]]), vec![1, 2, 3]);
    }

    #[test]
    fn round_trips_through_parts() {
        for len in [0usize, 1, 59 * 1024, 61 * 1024] {
            let bytes: Vec<u8> = (0..len)
                .map(|i| u8::try_from(i % 251).unwrap_or(0))
                .collect();
            let parts = room_sync::split_parts(&bytes, room_sync::PART_BYTES);
            assert_eq!(room_sync::join_parts(&parts), bytes);
        }
    }
}
