use crate::starboard;

#[must_use]
pub fn read_cell(
    canvas: &starboard::canvas::Canvas,
    pos: starboard::canvas::CellPos,
) -> starboard::canvas::Cell {
    let x = usize::from(pos.x);
    let y = usize::from(pos.y);
    if x >= starboard::canvas::CANVAS_SIDE || y >= starboard::canvas::CANVAS_SIDE {
        return starboard::canvas::Cell::default();
    }
    let tile_x = x.checked_div(starboard::canvas::TILE_SIDE).unwrap_or(0);
    let tile_y = y.checked_div(starboard::canvas::TILE_SIDE).unwrap_or(0);
    let tile_idx = tile_y
        .saturating_mul(starboard::canvas::TILES_PER_SIDE)
        .saturating_add(tile_x);
    let local_x = x.checked_rem(starboard::canvas::TILE_SIDE).unwrap_or(0);
    let local_y = y.checked_rem(starboard::canvas::TILE_SIDE).unwrap_or(0);
    let cell_idx = local_y
        .saturating_mul(starboard::canvas::TILE_SIDE)
        .saturating_add(local_x);

    canvas
        .tiles
        .get(tile_idx)
        .and_then(Option::as_ref)
        .and_then(|tile| tile.get(cell_idx).copied())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::read_cell;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut canvas = starboard::canvas::Canvas::default();
        let pos = starboard::canvas::CellPos { x: 10, y: 20 };
        assert_eq!(read_cell(&canvas, pos), starboard::canvas::Cell::default());
        let expected = starboard::canvas::Cell {
            lamport: 3,
            author: 9,
            color: Some(starboard::identity::PaletteColor(4)),
        };
        let _ = starboard::canvas::write_cell(&mut canvas, pos, expected);
        assert_eq!(read_cell(&canvas, pos), expected);
    }
}
