use crate::starboard;

pub fn brush_radius(size_index: u8) -> u16 {
    let clamped =
        usize::from(size_index).min(starboard::canvas::BRUSH_RADII.len().saturating_sub(1));
    starboard::canvas::BRUSH_RADII
        .get(clamped)
        .copied()
        .unwrap_or(starboard::canvas::BASE_BRUSH_RADIUS)
}

#[cfg(test)]
mod tests {
    use super::brush_radius;
    use crate::starboard;

    #[test]
    fn test_usage() {
        assert_eq!(brush_radius(0), 1);
        assert_eq!(
            brush_radius(starboard::canvas::DEFAULT_SIZE_INDEX),
            starboard::canvas::BASE_BRUSH_RADIUS
        );
        assert_eq!(brush_radius(8), 32);
        assert_eq!(brush_radius(99), 32);
        for (idx, &percent) in starboard::canvas::SIZE_PERCENTS.iter().enumerate() {
            let expected = (u32::from(starboard::canvas::BASE_BRUSH_RADIUS)
                .saturating_mul(u32::from(percent))
                / 100)
                .max(1);
            let idx_u8 = u8::try_from(idx).unwrap_or(0);
            assert_eq!(u32::from(brush_radius(idx_u8)), expected);
        }
    }
}
