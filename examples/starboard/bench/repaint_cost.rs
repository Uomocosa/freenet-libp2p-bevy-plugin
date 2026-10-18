use std::time::Instant;

use crate::starboard;

#[must_use]
pub fn repaint_cost(
    canvas: &mut starboard::canvas::Canvas,
    dirty_tiles: usize,
    rounds: u32,
) -> starboard::bench::RepaintCost {
    let side = starboard::canvas::CANVAS_SIDE;
    let mut pixels = vec![0_u8; side.saturating_mul(side).saturating_mul(4)];
    let started = Instant::now();
    for _ in 0..rounds {
        canvas.dirty_tiles = (0..dirty_tiles)
            .map(|index| starboard::canvas::TileId(u16::try_from(index).unwrap_or(0)))
            .collect();
        let _ = starboard::render::repaint_dirty_tiles(canvas, &mut pixels);
    }
    let elapsed = started.elapsed().as_micros();
    let rounds = u128::from(rounds.max(1));
    starboard::bench::RepaintCost {
        dirty_tiles,
        micros: u64::try_from(elapsed / rounds).unwrap_or(u64::MAX),
    }
}

#[cfg(test)]
mod tests {
    use super::repaint_cost;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut canvas = starboard::canvas::Canvas::default();
        let cell = starboard::canvas::Cell {
            lamport: 1,
            author: 1,
            color: Some(starboard::identity::PaletteColor(2)),
        };
        for index in 0..4_u16 {
            assert!(starboard::canvas::write_cell(
                &mut canvas,
                starboard::canvas::CellPos {
                    x: index
                        .saturating_mul(u16::try_from(starboard::canvas::TILE_SIDE).unwrap_or(1),),
                    y: 0,
                },
                cell,
            ));
        }

        let cost = repaint_cost(&mut canvas, 4, 16);
        assert_eq!(cost.dirty_tiles, 4);
        assert!(canvas.dirty_tiles.is_empty());
    }
}
