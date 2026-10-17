#[must_use]
pub fn split_parts(bytes: &[u8], part_bytes: usize) -> Vec<Vec<u8>> {
    if bytes.is_empty() {
        return vec![Vec::new()];
    }
    let width = part_bytes.max(1);
    bytes.chunks(width).map(<[u8]>::to_vec).collect()
}

#[cfg(test)]
mod tests {
    use crate::room_sync;

    #[test]
    fn test_usage() {
        assert_eq!(room_sync::split_parts(&[], 4), vec![Vec::<u8>::new()]);
        assert_eq!(room_sync::split_parts(&[1], 4), vec![vec![1]]);
        assert_eq!(
            room_sync::split_parts(&[1, 2, 3, 4, 5], 2),
            vec![vec![1, 2], vec![3, 4], vec![5]]
        );
    }
}
