use bevy::prelude::Image;
use bevy::render::render_resource::Extent3d;

use crate::starboard;

pub fn replace_pixels(image: &mut Image, next: starboard::render::BrushImage) {
    image.resize(Extent3d {
        width: next.side,
        height: next.side,
        depth_or_array_layers: 1,
    });
    image.data = Some(next.pixels);
}

#[cfg(test)]
mod tests {
    use super::replace_pixels;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut image = starboard::render::square_image(1, vec![0_u8; 4]);
        replace_pixels(
            &mut image,
            starboard::render::BrushImage {
                side: 2,
                pixels: vec![5_u8; 16],
            },
        );
        assert_eq!(image.texture_descriptor.size.width, 2);
        assert_eq!(image.data.as_deref(), Some(&[5_u8; 16][..]));
    }
}
