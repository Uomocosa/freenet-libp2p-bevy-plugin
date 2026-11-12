use crate::starboard;

pub fn merge_tile(
    canvas: &mut starboard::canvas::Canvas,
    id: starboard::canvas::TileId,
    remote: &starboard::canvas::Tile,
) {
    let tile_idx = usize::from(*id);
    if remote
        .iter()
        .all(|cell| *cell == starboard::canvas::Cell::default())
    {
        return;
    }
    let Some(tile_slot) = canvas.tiles.get_mut(tile_idx) else {
        return;
    };
    let tile = tile_slot.get_or_insert_with(|| Box::new(starboard::canvas::Tile::default()));
    let mut is_changed = false;
    for (local_cell, remote_cell) in tile.iter_mut().zip(remote.iter()) {
        if local_cell.absorb_cell(remote_cell) {
            is_changed = true;
        }
    }
    if is_changed {
        canvas.dirty_tiles.insert(id);
    }
}

#[cfg(test)]
mod tests {
    use super::merge_tile;
    use crate::starboard;
    use starboard::canvas::{FULL_COVERAGE, Layer};

    #[test]
    fn test_usage() {
        let mut canvas = starboard::canvas::Canvas::default();
        let mut remote = starboard::canvas::Tile::default();
        let top = Layer {
            rank: 5,
            author: 11,
            color: Some(starboard::identity::PaletteColor(3)),
            coverage: 128,
        };
        let under = Layer {
            rank: 4,
            author: 2,
            color: Some(starboard::identity::PaletteColor(1)),
            coverage: FULL_COVERAGE,
        };
        if let Some(slot) = remote.get_mut(0) {
            slot.top = top;
            slot.under = Some(starboard::canvas::UnderLayer::from(under));
        }
        merge_tile(&mut canvas, starboard::canvas::TileId(0), &remote);
        let cell = starboard::canvas::read_cell(&canvas, starboard::canvas::CellPos { x: 0, y: 0 });
        assert_eq!(cell.top, top);
        assert_eq!(cell.under, Some(starboard::canvas::UnderLayer::from(under)));
    }
}
