use crate::starboard;

pub fn rasterize(
    op: &starboard::canvas::DrawOp,
) -> impl Iterator<Item = starboard::canvas::CellPos> {
    let radius = starboard::canvas::brush_radius(op.size_index);
    let stride = usize::from((radius / 2).max(1));
    let centers = stamp_centers(op.from, op.to, stride);
    let r = i32::from(radius);
    let min_offset = 0_i32.saturating_sub(r);
    let shape = op.shape;
    let mut cells = Vec::new();
    for center in centers {
        let cx = i32::from(center.x);
        let cy = i32::from(center.y);
        for dy in min_offset..=r {
            for dx in min_offset..=r {
                if !starboard::canvas::is_inside_shape(shape, radius, dx, dy) {
                    continue;
                }
                let x = cx.saturating_add(dx);
                let y = cy.saturating_add(dy);
                if let (Ok(ux), Ok(uy)) = (u16::try_from(x), u16::try_from(y))
                    && usize::from(ux) < starboard::canvas::CANVAS_SIDE
                    && usize::from(uy) < starboard::canvas::CANVAS_SIDE
                {
                    cells.push(starboard::canvas::CellPos { x: ux, y: uy });
                }
            }
        }
    }
    cells.into_iter()
}

// needed helper: step along from -> to with integer Bresenham and emit stamp centers every stride steps
fn stamp_centers(
    from: starboard::canvas::CellPos,
    to: starboard::canvas::CellPos,
    stride: usize,
) -> Vec<starboard::canvas::CellPos> {
    let mut x = i32::from(from.x);
    let mut y = i32::from(from.y);
    let x1 = i32::from(to.x);
    let y1 = i32::from(to.y);
    let dx = (x1.saturating_sub(x)).abs();
    let sx = if x < x1 { 1_i32 } else { -1_i32 };
    let dy = 0_i32.saturating_sub((y1.saturating_sub(y)).abs());
    let sy = if y < y1 { 1_i32 } else { -1_i32 };
    let mut err = dx.saturating_add(dy);
    let mut step = 0_usize;
    let mut centers = Vec::new();

    loop {
        if (step.checked_rem(stride).unwrap_or(0) == 0 || (x == x1 && y == y1))
            && let (Ok(ux), Ok(uy)) = (u16::try_from(x), u16::try_from(y))
        {
            centers.push(starboard::canvas::CellPos { x: ux, y: uy });
        }
        if x == x1 && y == y1 {
            break;
        }
        let e2 = err.saturating_mul(2);
        if e2 >= dy {
            err = err.saturating_add(dy);
            x = x.saturating_add(sx);
        }
        if e2 <= dx {
            err = err.saturating_add(dx);
            y = y.saturating_add(sy);
        }
        step = step.saturating_add(1);
    }
    centers
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::rasterize;
    use crate::starboard;
    use crate::starboard::canvas::{CANVAS_SIDE, CellPos, DrawAction, DrawOp};
    use crate::starboard::identity::{PaletteColor, SHAPES, Shape};

    #[test]
    fn test_usage() {
        for shape in SHAPES {
            let click_min_size = DrawOp {
                action: DrawAction::Paint(PaletteColor(0)),
                shape,
                size_index: 0,
                from: CellPos { x: 100, y: 100 },
                to: CellPos { x: 100, y: 100 },
            };
            let covered: BTreeSet<_> = rasterize(&click_min_size).collect();
            assert!(!covered.is_empty());
            assert!(covered.contains(&CellPos { x: 100, y: 100 }));
        }
    }

    #[test]
    fn segment_has_no_gaps_and_clips_at_canvas_borders() {
        let stroke = DrawOp {
            action: DrawAction::Paint(PaletteColor(1)),
            shape: Shape::Circle,
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
            from: CellPos { x: 20, y: 20 },
            to: CellPos { x: 120, y: 20 },
        };
        let covered: BTreeSet<_> = rasterize(&stroke).collect();
        for x in 20_u16..=120 {
            assert!(
                covered.contains(&CellPos { x, y: 20 }),
                "missing centerline cell at x={x}"
            );
        }

        let max_coord = u16::try_from(CANVAS_SIDE.saturating_sub(1)).unwrap_or(1023);
        let border_op = DrawOp {
            action: DrawAction::Erase,
            shape: Shape::Square,
            size_index: 8,
            from: CellPos { x: 0, y: 0 },
            to: CellPos { x: max_coord, y: 0 },
        };
        let border_cells: Vec<_> = rasterize(&border_op).collect();
        assert!(!border_cells.is_empty());
        assert!(
            border_cells.iter().all(|pos| {
                usize::from(pos.x) < CANVAS_SIDE && usize::from(pos.y) < CANVAS_SIDE
            })
        );
    }
}
