use serde::{Deserialize, Serialize};

use crate::starboard;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Layer {
    pub rank: u32,
    pub author: u16,
    pub color: Option<starboard::identity::PaletteColor>,
    pub coverage: u8,
}

impl Layer {
    #[must_use]
    pub const fn key(&self) -> starboard::canvas::LayerKey {
        starboard::canvas::LayerKey {
            rank: self.rank,
            author: self.author,
            color: self.color,
        }
    }

    #[must_use]
    pub const fn is_full(&self) -> bool {
        self.coverage == starboard::canvas::FULL_COVERAGE
    }
}

#[cfg(test)]
mod tests {
    use super::Layer;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let layer = Layer {
            rank: 4,
            author: 2,
            color: Some(starboard::identity::PaletteColor(3)),
            coverage: starboard::canvas::FULL_COVERAGE,
        };
        assert!(layer.is_full());
        assert_eq!(layer.key().rank, 4);
        let erase = starboard::canvas::LayerKey {
            rank: 4,
            author: 2,
            color: None,
        };
        assert!(layer.key() > erase);
    }
}
