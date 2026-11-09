use crate::starboard;

#[must_use]
pub fn solid_brush_image(
    shape: starboard::identity::Shape,
    radius: u16,
    color: starboard::identity::PaletteColor,
) -> starboard::render::BrushImage {
    let rgba = starboard::render::palette_rgba(color);
    starboard::render::brush_image(shape, radius, rgba, rgba)
}

#[cfg(test)]
mod tests {
    use super::solid_brush_image;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let color = starboard::identity::PaletteColor(3);
        let image = solid_brush_image(starboard::identity::Shape::Square, 1, color);
        let rgba = starboard::render::palette_rgba(color);
        assert!(image.pixels.chunks_exact(4).all(|pixel| pixel == rgba));
    }
}
