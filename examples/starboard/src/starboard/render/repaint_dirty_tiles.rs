use crate::starboard;

#[must_use]
pub fn repaint_dirty_tiles(canvas: &mut starboard::canvas::Canvas, pixels: &mut [u8]) -> usize {
    let dirty = std::mem::take(&mut canvas.dirty_tiles);
    let mut painted = 0_usize;
    for id in dirty {
        if starboard::render::paint_tile(canvas, id, pixels) {
            painted = painted.saturating_add(1);
        }
    }
    painted
}

#[cfg(test)]
mod tests {
    use super::repaint_dirty_tiles;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut canvas = starboard::canvas::Canvas::default();
        let cell = starboard::canvas::Cell {
            lamport: 1,
            author: 3,
            color: Some(starboard::identity::PaletteColor(4)),
        };
        assert!(starboard::canvas::write_cell(
            &mut canvas,
            starboard::canvas::CellPos { x: 40, y: 40 },
            cell
        ));
        let mut pixels = vec![0_u8; 1024 * 1024 * 4];

        assert_eq!(repaint_dirty_tiles(&mut canvas, &mut pixels), 1);
        assert!(canvas.dirty_tiles.is_empty());
        assert_eq!(repaint_dirty_tiles(&mut canvas, &mut pixels), 0);
    }
}
