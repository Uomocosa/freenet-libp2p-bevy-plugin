use bevy::prelude::Resource;

use crate::starboard;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct OwnBrush {
    pub shape: starboard::identity::Shape,
    pub color: starboard::identity::PaletteColor,
    pub size_index: u8,
}

impl Default for OwnBrush {
    fn default() -> Self {
        Self {
            shape: starboard::identity::Shape::Circle,
            color: starboard::identity::PaletteColor(0),
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::OwnBrush;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let brush = OwnBrush::default();
        assert_eq!(brush.shape, starboard::identity::Shape::Circle);
        assert_eq!(brush.color, starboard::identity::PaletteColor(0));
        assert_eq!(brush.size_index, starboard::canvas::DEFAULT_SIZE_INDEX);
    }
}
