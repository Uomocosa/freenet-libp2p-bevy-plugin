use crate::starboard;

#[must_use]
pub fn encode_snapshot(
    tiles: &[starboard::canvas::TouchedTile],
) -> starboard::canvas::CanvasSnapshot {
    let raw = bincode::serialize(tiles).unwrap_or_default();
    let compressed = zstd::encode_all(raw.as_slice(), 3).unwrap_or_default();
    starboard::canvas::CanvasSnapshot(compressed)
}

#[cfg(test)]
mod tests {
    use super::encode_snapshot;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let touched = vec![starboard::canvas::TouchedTile {
            id: starboard::canvas::TileId(3),
            cells: vec![starboard::canvas::Cell::default(); starboard::canvas::CELLS_PER_TILE],
        }];
        let snapshot = encode_snapshot(&touched);
        assert!(!snapshot.is_empty());
        assert_eq!(starboard::canvas::decode_snapshot(&snapshot), touched);
    }
}
