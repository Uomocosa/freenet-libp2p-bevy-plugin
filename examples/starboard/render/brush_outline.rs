use crate::starboard;

#[must_use]
pub fn brush_outline(
    shape: starboard::identity::Shape,
    radius: u16,
    color: starboard::identity::PaletteColor,
) -> starboard::render::BrushOutline {
    let side = radius.saturating_mul(2).saturating_add(1);
    let mut pixels = vec![
        0_u8;
        usize::from(side)
            .saturating_mul(usize::from(side))
            .saturating_mul(4)
    ];
    for y in 0_u16..side {
        for x in 0_u16..side {
            let dx = i32::from(x).saturating_sub(i32::from(radius));
            let dy = i32::from(y).saturating_sub(i32::from(radius));
            if !starboard::canvas::is_inside_shape(shape, radius, dx, dy) {
                continue;
            }
            if is_interior(shape, radius, dx, dy) {
                continue;
            }
            let start = usize::from(y)
                .saturating_mul(usize::from(side))
                .saturating_add(usize::from(x))
                .saturating_mul(4);
            if let Some(pixel) = pixels.get_mut(start..start.saturating_add(4)) {
                pixel.copy_from_slice(&starboard::render::palette_rgba(color));
            }
        }
    }
    starboard::render::BrushOutline {
        side: u32::from(side),
        pixels,
    }
}

// needed helper: an outline pixel is inside the shape but has a neighbour outside it
fn is_interior(shape: starboard::identity::Shape, radius: u16, dx: i32, dy: i32) -> bool {
    [(1_i32, 0_i32), (-1, 0), (0, 1), (0, -1)]
        .iter()
        .all(|(step_x, step_y)| {
            starboard::canvas::is_inside_shape(
                shape,
                radius,
                dx.saturating_add(*step_x),
                dy.saturating_add(*step_y),
            )
        })
}

#[cfg(test)]
mod tests {
    use super::brush_outline;
    use crate::starboard;

    fn pixel_at(outline: &starboard::render::BrushOutline, x: u32, y: u32) -> [u8; 4] {
        let side = usize::try_from(outline.side).expect("side fits usize");
        let start = (usize::try_from(y).expect("y fits usize") * side
            + usize::try_from(x).expect("x fits usize"))
            * 4;
        let mut pixel = [0_u8; 4];
        pixel.copy_from_slice(&outline.pixels[start..start + 4]);
        pixel
    }

    #[test]
    fn test_usage() {
        let color = starboard::identity::PaletteColor(2);
        let outline = brush_outline(starboard::identity::Shape::Square, 2, color);
        assert_eq!(outline.side, 5);
        assert_eq!(outline.pixels.len(), 5 * 5 * 4);
        assert_eq!(
            pixel_at(&outline, 0, 2),
            starboard::render::palette_rgba(color)
        );
        assert_eq!(pixel_at(&outline, 2, 2), [0, 0, 0, 0]);
    }

    #[test]
    fn every_shape_has_an_outline_at_the_smallest_radius() {
        let color = starboard::identity::PaletteColor(1);
        for shape in starboard::identity::SHAPES {
            let outline = brush_outline(shape, 1, color);
            assert_eq!(outline.side, 3);
            assert!(
                outline
                    .pixels
                    .chunks_exact(4)
                    .any(|pixel| pixel == starboard::render::palette_rgba(color)),
                "{shape:?} has no outline pixel"
            );
        }
    }
}
