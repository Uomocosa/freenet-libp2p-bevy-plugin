use crate::starboard;

#[must_use]
pub fn hash_canvas(canvas: &starboard::canvas::Canvas) -> u64 {
    let mut hasher = blake3::Hasher::new();
    for (idx, maybe_tile) in canvas.tiles.iter().enumerate() {
        let Some(tile) = maybe_tile else {
            continue;
        };
        if tile
            .iter()
            .all(|cell| *cell == starboard::canvas::Cell::default())
        {
            continue;
        }
        let tile_id = u16::try_from(idx).unwrap_or(0);
        let slice_hash = starboard::canvas::hash_tile(tile);
        hasher.update(&tile_id.to_be_bytes());
        hasher.update(&slice_hash.to_be_bytes());
    }
    let digest = hasher.finalize();
    let mut leading = [0_u8; 8];
    for (dst, src) in leading.iter_mut().zip(digest.as_bytes()) {
        *dst = *src;
    }
    u64::from_be_bytes(leading)
}

#[cfg(test)]
mod tests {
    use super::hash_canvas;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut canvas = starboard::canvas::Canvas::default();
        let empty_hash = hash_canvas(&canvas);
        let _ = starboard::canvas::write_cell(
            &mut canvas,
            starboard::canvas::CellPos { x: 4, y: 4 },
            starboard::canvas::Cell {
                lamport: 1,
                author: 1,
                color: Some(starboard::identity::PaletteColor(0)),
            },
        );
        assert_ne!(hash_canvas(&canvas), empty_hash);
    }
}
