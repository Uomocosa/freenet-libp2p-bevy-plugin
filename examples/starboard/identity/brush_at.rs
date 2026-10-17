use crate::starboard;

#[must_use]
pub fn brush_at(slot: usize) -> starboard::identity::DailyBrush {
    let wrapped = slot
        .checked_rem(starboard::identity::COMBINATION_COUNT)
        .unwrap_or(0);
    let palette_len = starboard::identity::PALETTE.len();
    let shape_index = wrapped.checked_div(palette_len).unwrap_or(0);
    let color_index = u8::try_from(wrapped.checked_rem(palette_len).unwrap_or(0)).unwrap_or(0);
    let shape = starboard::identity::SHAPES
        .get(shape_index)
        .copied()
        .unwrap_or(starboard::identity::Shape::Circle);
    starboard::identity::DailyBrush {
        shape,
        color: starboard::identity::PaletteColor(color_index),
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::brush_at;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let distinct: BTreeSet<_> = (0..starboard::identity::COMBINATION_COUNT)
            .map(brush_at)
            .collect();
        assert_eq!(distinct.len(), starboard::identity::COMBINATION_COUNT);
        assert_eq!(
            brush_at(0),
            brush_at(starboard::identity::COMBINATION_COUNT)
        );
    }
}
