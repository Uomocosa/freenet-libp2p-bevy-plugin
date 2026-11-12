use freenet_libp2p_bevy_plugin::room_sync;

use crate::starboard;

#[must_use]
pub fn hash_tile(tile: &starboard::canvas::Tile) -> room_sync::ReplicaSliceContentHash {
    let mut bytes = Vec::with_capacity(starboard::canvas::CELLS_PER_TILE.saturating_mul(24));
    for cell in tile.iter() {
        bytes.extend_from_slice(&bincode::serialize(cell).unwrap_or_default());
    }
    room_sync::ReplicaSliceContentHash::from(bytes.as_slice())
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
            slot.top = starboard::canvas::Layer {
                rank: 1,
                author: 2,
                color: Some(starboard::identity::PaletteColor(5)),
                mask: 200,
            };
        }
        assert_eq!(hash_tile(&empty), hash_tile(&empty));
        assert_ne!(hash_tile(&empty), hash_tile(&painted));
    }
}
