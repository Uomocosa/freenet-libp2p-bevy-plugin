use derive_more::{Deref, DerefMut};

use crate::starboard;

#[derive(Debug, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct Tile(pub [starboard::canvas::Cell; starboard::canvas::CELLS_PER_TILE]);

impl Default for Tile {
    fn default() -> Self {
        Self(std::array::from_fn(|_| starboard::canvas::Cell::default()))
    }
}

impl Tile {
    pub(crate) fn try_from_cells(cells: &[starboard::canvas::Cell]) -> Option<Self> {
        if cells.len() != starboard::canvas::CELLS_PER_TILE {
            return None;
        }
        let mut tile = Self::default();
        for (dst, src) in tile.iter_mut().zip(cells.iter()) {
            *dst = *src;
        }
        Some(tile)
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
