use crate::starboard;

#[must_use]
pub fn palette_rgba(color: starboard::identity::PaletteColor) -> [u8; 4] {
    starboard::identity::PALETTE
        .get(usize::from(*color))
        .map_or([0, 0, 0, 0], |[red, green, blue]| {
            [*red, *green, *blue, 255]
        })
}

#[cfg(test)]
mod tests {
    use super::palette_rgba;
    use crate::starboard;

    #[test]
    fn test_usage() {
        assert_eq!(
            palette_rgba(starboard::identity::PaletteColor(0)),
            [230, 57, 70, 255]
        );
        assert_eq!(
            palette_rgba(starboard::identity::PaletteColor(200)),
            [0, 0, 0, 0]
        );
    }
}
