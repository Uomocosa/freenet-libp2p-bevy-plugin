use freenet_libp2p_bevy_plugin::room_sync;

use crate::starboard;

#[must_use]
pub fn hash_tile(tile: &starboard::canvas::Tile) -> room_sync::ReplicaSliceContentHash {
    let mut bytes = Vec::with_capacity(starboard::canvas::CELLS_PER_TILE.saturating_mul(7));
    for cell in tile.iter() {
        bytes.extend_from_slice(&cell.lamport.to_be_bytes());
        bytes.extend_from_slice(&cell.author.to_be_bytes());
        let color_tag = cell.color.map_or(255_u8, |palette_color| *palette_color);
        bytes.push(color_tag);
    }
    room_sync::ReplicaSliceContentHash::from_bytes(&bytes)
}

#[cfg(test)]
mod tests {
    use super::hash_tile;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let empty = starboard::canvas::Tile::default();
        let mut painted = starboard::canvas::Tile::default();
        if let Some(slot) = painted.get_mut(10) {
            *slot = starboard::canvas::Cell {
                lamport: 1,
                author: 2,
                color: Some(starboard::identity::PaletteColor(5)),
            };
        }
        assert_eq!(hash_tile(&empty), hash_tile(&empty));
        assert_ne!(hash_tile(&empty), hash_tile(&painted));
    }
}
