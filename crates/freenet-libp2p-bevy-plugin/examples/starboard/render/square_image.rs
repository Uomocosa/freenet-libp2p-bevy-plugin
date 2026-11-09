use bevy::asset::RenderAssetUsages;
use bevy::prelude::Image;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

#[must_use]
pub fn square_image(side: u32, pixels: Vec<u8>) -> Image {
    Image::new(
        Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
}

#[cfg(test)]
mod tests {
    use super::square_image;

    #[test]
    fn test_usage() {
        let image = square_image(2, vec![9_u8; 2 * 2 * 4]);
        assert_eq!(image.texture_descriptor.size.width, 2);
        assert_eq!(image.data.as_deref(), Some(&[9_u8; 16][..]));
    }
}
