use serde::{Deserialize, Serialize};

use crate::starboard;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UnderLayer {
    pub rank: u32,
    pub author: u16,
    pub color: Option<starboard::identity::PaletteColor>,
}

impl From<starboard::canvas::Layer> for UnderLayer {
    fn from(layer: starboard::canvas::Layer) -> Self {
        Self {
            rank: layer.rank,
            author: layer.author,
            color: layer.color,
        }
    }
}

impl UnderLayer {
    #[must_use]
    pub const fn key(self) -> starboard::canvas::LayerKey {
        starboard::canvas::LayerKey {
            rank: self.rank,
            author: self.author,
            color: self.color,
        }
    }

    #[must_use]
    pub(crate) const fn as_full_layer(self) -> starboard::canvas::Layer {
        starboard::canvas::Layer {
            rank: self.rank,
            author: self.author,
            color: self.color,
            coverage: starboard::canvas::FULL_COVERAGE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UnderLayer;

    #[test]
    fn test_usage() {
        let under = UnderLayer {
            rank: 2,
            author: 1,
            color: None,
        };
        assert!(under.as_full_layer().is_full());
        assert_eq!(UnderLayer::from(under.as_full_layer()), under);
        assert_eq!(under.key().rank, 2);
    }
}
