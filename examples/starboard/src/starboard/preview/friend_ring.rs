use crate::starboard;

#[must_use]
pub fn friend_ring(side: u16, band: u16) -> starboard::render::BrushImage {
    let radius = side.checked_div(2).unwrap_or(0);
    let inner = radius.saturating_sub(band);
    let pixels = (0..side)
        .flat_map(|ty| (0..side).map(move |tx| (tx, ty)))
        .flat_map(|(tx, ty)| {
            let dx = i32::from(tx).saturating_sub(i32::from(radius));
            let dy = i32::from(ty).saturating_sub(i32::from(radius));
            let is_inside_ring = starboard::canvas::is_inside_shape(
                starboard::identity::Shape::Circle,
                radius,
                dx,
                dy,
            ) && !starboard::canvas::is_inside_shape(
                starboard::identity::Shape::Circle,
                inner,
                dx,
                dy,
            );
            if is_inside_ring {
                starboard::preview::FRIEND_HIGHLIGHT_RGBA
            } else {
                [0, 0, 0, 0]
            }
        })
        .collect();
    starboard::render::BrushImage {
        side: u32::from(side),
        pixels,
    }
}

#[cfg(test)]
mod tests {
    use super::friend_ring;
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
        let side = starboard::preview::FRIEND_HIGHLIGHT_RING_PX;
        let half = side.checked_div(2).unwrap_or(0);
        let image = friend_ring(side, starboard::preview::FRIEND_HIGHLIGHT_BAND_PX);

        assert_eq!(image.side, u32::from(side));
        assert_eq!(
            image.pixels.len(),
            usize::from(side)
                .saturating_mul(usize::from(side))
                .saturating_mul(4)
        );
        assert_eq!(
            pixel_at(&image, u32::from(half), 0),
            starboard::preview::FRIEND_HIGHLIGHT_RGBA
        );
        assert_eq!(
            pixel_at(&image, u32::from(half), u32::from(half)),
            [0, 0, 0, 0]
        );
    }
}
