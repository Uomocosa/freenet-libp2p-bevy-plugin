use crate::starboard;

pub fn write_cell(
    canvas: &mut starboard::canvas::Canvas,
    pos: starboard::canvas::CellPos,
    cell: starboard::canvas::Cell,
) -> bool {
    let x = usize::from(pos.x);
    let y = usize::from(pos.y);
    if x >= starboard::canvas::CANVAS_SIDE || y >= starboard::canvas::CANVAS_SIDE {
        return false;
    }
    if cell <= starboard::canvas::Cell::default() {
        return false;
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

    let Some(tile_slot) = canvas.tiles.get_mut(tile_idx) else {
        return false;
    };
    let tile = tile_slot.get_or_insert_with(|| Box::new(starboard::canvas::Tile::default()));
    let Some(existing) = tile.get_mut(cell_idx) else {
        return false;
    };
    if cell > *existing {
        *existing = cell;
        let tile_id = starboard::canvas::TileId(u16::try_from(tile_idx).unwrap_or(0));
        canvas.dirty_tiles.insert(tile_id);
        true
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::write_cell;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut canvas = starboard::canvas::Canvas::default();
        let pos = starboard::canvas::CellPos { x: 33, y: 65 };
        let older = starboard::canvas::Cell {
            lamport: 1,
            author: 10,
            color: Some(starboard::identity::PaletteColor(2)),
        };
        let newer = starboard::canvas::Cell {
            lamport: 2,
            author: 5,
            color: None,
        };
        assert!(write_cell(&mut canvas, pos, older));
        assert!(write_cell(&mut canvas, pos, newer));
        assert!(!write_cell(&mut canvas, pos, older));
        assert_eq!(starboard::canvas::read_cell(&canvas, pos), newer);
        assert!(canvas.dirty_tiles.contains(&starboard::canvas::TileId(65)));
    }

    #[test]
    fn equal_lamport_and_author_tie_breaks_deterministically_by_color() {
        let pos = starboard::canvas::CellPos { x: 5, y: 5 };
        let low_color = starboard::canvas::Cell {
            lamport: 7,
            author: 42,
            color: Some(starboard::identity::PaletteColor(1)),
        };
        let high_color = starboard::canvas::Cell {
            lamport: 7,
            author: 42,
            color: Some(starboard::identity::PaletteColor(9)),
        };
        let erased = starboard::canvas::Cell {
            lamport: 7,
            author: 42,
            color: None,
        };

        let mut first = starboard::canvas::Canvas::default();
        let _ = write_cell(&mut first, pos, low_color);
        let _ = write_cell(&mut first, pos, high_color);
        let _ = write_cell(&mut first, pos, erased);

        let mut second = starboard::canvas::Canvas::default();
        let _ = write_cell(&mut second, pos, erased);
        let _ = write_cell(&mut second, pos, high_color);
        let _ = write_cell(&mut second, pos, low_color);

        assert_eq!(starboard::canvas::read_cell(&first, pos), high_color);
        assert_eq!(
            starboard::canvas::read_cell(&first, pos),
            starboard::canvas::read_cell(&second, pos)
        );
    }
}
