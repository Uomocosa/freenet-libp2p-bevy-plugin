use crate::starboard;

#[must_use]
pub fn outline_brush(
    shape: starboard::identity::Shape,
    radius: u16,
    scale: f32,
) -> starboard::render::BrushOutline {
    let texels_per_cell = ceil_to_u16(scale).max(1);
    let band = ceil_to_u16(
        f32::from(starboard::render::OWN_OUTLINE_PX) * f32::from(texels_per_cell)
            / scale.max(f32::EPSILON),
    )
    .max(1);
    let cells = radius.saturating_mul(2).saturating_add(1);
    let side = cells
        .saturating_mul(texels_per_cell)
        .saturating_add(band.saturating_mul(2));
    let is_inside = |tx: i32, ty: i32| {
        let cell = |texel: i32| {
            texel
                .saturating_sub(i32::from(band))
                .checked_div_euclid(i32::from(texels_per_cell))
                .unwrap_or(i32::MIN)
                .saturating_sub(i32::from(radius))
        };
        starboard::canvas::is_inside_shape(shape, radius, cell(tx), cell(ty))
    };
    let pixels = (0..side)
        .flat_map(|ty| (0..side).map(move |tx| (i32::from(tx), i32::from(ty))))
        .flat_map(|(tx, ty)| {
            let is_outline = !is_inside(tx, ty)
                && (1..=i32::from(band)).any(|step| {
                    is_inside(tx.saturating_add(step), ty)
                        || is_inside(tx.saturating_sub(step), ty)
                        || is_inside(tx, ty.saturating_add(step))
                        || is_inside(tx, ty.saturating_sub(step))
                });
            if is_outline {
                starboard::render::OWN_OUTLINE_RGBA
            } else {
                [0, 0, 0, 0]
            }
        })
        .collect();
    starboard::render::BrushOutline {
        image: starboard::render::BrushImage {
            side: u32::from(side),
            pixels,
        },
        screen_side: f32::from(side) * scale / f32::from(texels_per_cell),
    }
}

// needed helper: as_conversions is denied, so round a small positive length up by search
fn ceil_to_u16(length: f32) -> u16 {
    (0..u16::MAX)
        .find(|candidate| f32::from(*candidate) >= length)
        .unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests {
    use super::outline_brush;
    use crate::starboard;

    fn pixel_at(image: &starboard::render::BrushImage, x: usize, y: usize) -> [u8; 4] {
        let side = usize::try_from(image.side).expect("side fits usize");
        let start = (y * side + x) * 4;
        let mut pixel = [0_u8; 4];
        pixel.copy_from_slice(&image.pixels[start..start + 4]);
        pixel
    }

    #[test]
    fn test_usage() {
        let outline = outline_brush(starboard::identity::Shape::Square, 1, 1.0);
        assert_eq!(outline.image.side, 7);
        assert!((outline.screen_side - 7.0).abs() < 0.000_1);
        assert_eq!(
            pixel_at(&outline.image, 0, 3),
            starboard::render::OWN_OUTLINE_RGBA
        );
        assert_eq!(pixel_at(&outline.image, 3, 3), [0, 0, 0, 0]);
    }

    #[test]
    fn outline_band_stays_about_two_screen_pixels_when_zoomed() {
        let outline = outline_brush(starboard::identity::Shape::Square, 1, 4.0);
        assert_eq!(outline.image.side, 3 * 4 + 2 * 2);
        assert!((outline.screen_side - 16.0).abs() < 0.000_1);
    }
}
