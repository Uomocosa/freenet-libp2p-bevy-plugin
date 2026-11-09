use derive_more::{Deref, DerefMut};

use crate::starboard;

#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct Tile(pub [starboard::canvas::Cell; starboard::canvas::CELLS_PER_TILE]);

impl Default for Tile {
    fn default() -> Self {
        Self([starboard::canvas::Cell::default(); starboard::canvas::CELLS_PER_TILE])
    }
}

#[cfg(test)]
mod tests {
    use super::Tile;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let tile = Tile::default();
        assert_eq!(tile.len(), starboard::canvas::CELLS_PER_TILE);
        assert!(
            tile.iter()
                .all(|cell| *cell == starboard::canvas::Cell::default())
        );
    }
}
