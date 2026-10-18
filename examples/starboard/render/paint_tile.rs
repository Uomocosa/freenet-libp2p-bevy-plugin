use crate::starboard;

#[must_use]
pub fn paint_tile(
    canvas: &starboard::canvas::Canvas,
    id: starboard::canvas::TileId,
    pixels: &mut [u8],
) -> bool {
    let side = starboard::canvas::CANVAS_SIDE;
    if pixels.len() < side.saturating_mul(side).saturating_mul(4) {
        return false;
    }
    let tile_idx = usize::from(*id);
    let Some(Some(tile)) = canvas.tiles.get(tile_idx) else {
        return false;
    };
    let tile_x = tile_idx
        .checked_rem(starboard::canvas::TILES_PER_SIDE)
        .unwrap_or(0);
    let tile_y = tile_idx
        .checked_div(starboard::canvas::TILES_PER_SIDE)
        .unwrap_or(0);
    for (cell_idx, cell) in tile.iter().enumerate() {
        let local_x = cell_idx
            .checked_rem(starboard::canvas::TILE_SIDE)
            .unwrap_or(0);
        let local_y = cell_idx
            .checked_div(starboard::canvas::TILE_SIDE)
            .unwrap_or(0);
        let x = tile_x
            .saturating_mul(starboard::canvas::TILE_SIDE)
            .saturating_add(local_x);
        let y = tile_y
            .saturating_mul(starboard::canvas::TILE_SIDE)
            .saturating_add(local_y);
        let start = y.saturating_mul(side).saturating_add(x).saturating_mul(4);
        if let Some(pixel) = pixels.get_mut(start..start.saturating_add(4)) {
            pixel.copy_from_slice(&cell.color.map_or(
                starboard::render::CANVAS_BACKGROUND,
                starboard::render::palette_rgba,
            ));
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::paint_tile;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut canvas = starboard::canvas::Canvas::default();
        let color = starboard::identity::PaletteColor(0);
        let cell = starboard::canvas::Cell {
            lamport: 1,
            author: 7,
            color: Some(color),
        };
        assert!(starboard::canvas::write_cell(
            &mut canvas,
            starboard::canvas::CellPos { x: 3, y: 4 },
            cell
        ));

        let mut pixels = vec![0_u8; 1024 * 1024 * 4];
        assert!(paint_tile(
            &canvas,
            starboard::canvas::TileId(0),
            &mut pixels
        ));
        let start = (4 * 1024 + 3) * 4;
        assert_eq!(
            &pixels[start..start + 4],
            &starboard::render::palette_rgba(color)
        );
        assert_eq!(&pixels[..4], &starboard::render::CANVAS_BACKGROUND);
        assert!(!paint_tile(
            &canvas,
            starboard::canvas::TileId(5),
            &mut pixels
        ));
        assert!(!paint_tile(&canvas, starboard::canvas::TileId(0), &mut []));
    }
}
