use crate::starboard;

#[must_use]
pub fn blend_cell_rgba(cell: &starboard::canvas::Cell, x: usize, y: usize) -> [u8; 4] {
    let background = starboard::render::checker_background_rgba_at(x, y);
    let top = cell
        .top
        .color
        .map_or(background, starboard::render::palette_rgba);
    let base = cell.under.map_or(background, |under| {
        under
            .color
            .map_or(background, starboard::render::palette_rgba)
    });
    let alpha = u32::from(cell.top.coverage);
    let mut blended = [0_u8; 4];
    for (index, channel) in blended.iter_mut().enumerate() {
        let top_channel = u32::from(top.get(index).copied().unwrap_or(0));
        let base_channel = u32::from(base.get(index).copied().unwrap_or(0));
        let mixed = top_channel
            .saturating_mul(alpha)
            .saturating_add(base_channel.saturating_mul(255_u32.saturating_sub(alpha)))
            .saturating_add(127);
        *channel = u8::try_from(mixed.checked_div(255).unwrap_or(0)).unwrap_or(0);
    }
    blended
}

#[cfg(test)]
mod tests {
    use super::blend_cell_rgba;
    use crate::starboard;
    use starboard::canvas::{Cell, FULL_COVERAGE, Layer, UnderLayer};
    use starboard::identity::PaletteColor;

    #[test]
    fn test_usage() {
        let red = PaletteColor(0);
        let blue = PaletteColor(6);
        let background = starboard::render::checker_background_rgba_at(0, 0);
        assert_eq!(blend_cell_rgba(&Cell::default(), 0, 0), background);

        let solid = Cell {
            top: Layer {
                rank: 1,
                author: 1,
                color: Some(red),
                coverage: FULL_COVERAGE,
            },
            under: None,
        };
        assert_eq!(
            blend_cell_rgba(&solid, 0, 0),
            starboard::render::palette_rgba(red)
        );

        let half_over_background = Cell {
            top: Layer {
                coverage: 128,
                ..solid.top
            },
            under: None,
        };
        let edge = blend_cell_rgba(&half_over_background, 0, 0);
        let expected_red = u32::from(starboard::render::palette_rgba(red)[0])
            .saturating_add(u32::from(background[0]))
            .checked_div(2)
            .unwrap_or(0);
        assert!(u32::from(edge[0]).abs_diff(expected_red) <= 1);

        let over_blue = Cell {
            top: half_over_background.top,
            under: Some(UnderLayer::from(Layer {
                rank: 0,
                author: 0,
                color: Some(blue),
                coverage: FULL_COVERAGE,
            })),
        };
        let mixed = blend_cell_rgba(&over_blue, 0, 0);
        assert_ne!(mixed, edge);
        assert_eq!(mixed[3], 255);
    }
}
