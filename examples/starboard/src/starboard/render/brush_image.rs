use crate::starboard;

#[must_use]
pub fn brush_image(
    shape: starboard::identity::Shape,
    radius: u16,
    fill: [u8; 4],
    edge: [u8; 4],
) -> starboard::render::BrushImage {
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
            let rgba = if shape.is_interior(radius, dx, dy) {
                fill
            } else {
                edge
            };
            let start = usize::from(y)
                .saturating_mul(usize::from(side))
                .saturating_add(usize::from(x))
                .saturating_mul(4);
            if let Some(pixel) = pixels.get_mut(start..start.saturating_add(4)) {
                pixel.copy_from_slice(&rgba);
            }
        }
    }
    starboard::render::BrushImage {
        side: u32::from(side),
        pixels,
    }
}

#[cfg(test)]
mod tests {
    use super::brush_image;
    use crate::starboard;

    fn pixel_at(image: &starboard::render::BrushImage, x: u32, y: u32) -> [u8; 4] {
        let side = usize::try_from(image.side).expect("side fits usize");
        let row = usize::try_from(y)
            .expect("y fits usize")
            .checked_mul(side)
            .expect("row fits usize");
        let start = row
            .checked_add(usize::try_from(x).expect("x fits usize"))
            .and_then(|offset| offset.checked_mul(4))
            .expect("pixel offset fits usize");
        let mut pixel = [0_u8; 4];
        pixel.copy_from_slice(
            &image.pixels[start..start.checked_add(4).expect("pixel end fits usize")],
        );
        pixel
    }

    #[test]
    fn test_usage() {
        let (fill, edge) = ([1, 2, 3, 128], [1, 2, 3, 255]);
        let image = brush_image(starboard::identity::Shape::Square, 2, fill, edge);
        assert_eq!(image.side, 5);
        assert_eq!(image.pixels.len(), 5 * 5 * 4);
        assert_eq!(pixel_at(&image, 0, 2), edge);
        assert_eq!(pixel_at(&image, 0, 0), edge);
        assert_eq!(pixel_at(&image, 2, 2), fill);
        assert_eq!(pixel_at(&image, 1, 1), fill);
    }

    #[test]
    fn every_shape_has_an_edge_at_the_smallest_radius() {
        let edge = [7, 7, 7, 255];
        for shape in starboard::identity::SHAPES {
            let image = brush_image(shape, 1, [0, 0, 0, 0], edge);
            assert_eq!(image.side, 3);
            assert!(
                image.pixels.as_chunks::<4>().0.contains(&edge),
                "{shape:?} has no edge pixel"
            );
        }
    }
}
