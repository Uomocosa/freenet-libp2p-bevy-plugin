use crate::starboard;

#[must_use]
pub fn decode_snapshot(
    snapshot: &starboard::canvas::CanvasSnapshot,
) -> Vec<starboard::canvas::TouchedTile> {
    let Ok(decompressed) = zstd::decode_all(snapshot.as_slice()) else {
        return Vec::new();
    };
    bincode::deserialize(&decompressed).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::decode_snapshot;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let touched = vec![starboard::canvas::TouchedTile {
            id: starboard::canvas::TileId(7),
            cells: vec![
                starboard::canvas::Cell {
                    lamport: 4,
                    author: 8,
                    color: Some(starboard::identity::PaletteColor(2)),
                };
                starboard::canvas::CELLS_PER_TILE
            ],
        }];
        let encoded = starboard::canvas::encode_snapshot(&touched);
        assert_eq!(decode_snapshot(&encoded), touched);
    }
}
