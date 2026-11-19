use bevy::prelude::Image;

#[must_use]
pub fn test_pattern_image() -> Image {
    let palette: [[u8; 4]; 6] = [
        [38, 70, 96, 255],
        [52, 92, 122, 255],
        [66, 114, 148, 255],
        [82, 138, 176, 255],
        [38, 70, 96, 255],
        [24, 48, 68, 255],
    ];
    let row: Vec<u8> = palette.concat().into_iter().cycle().take(640).collect();
    let data: Vec<u8> = row.iter().copied().cycle().take(64_000).collect();
    Image::new(
        bevy::render::render_resource::Extent3d {
            width: 160,
            height: 100,
            depth_or_array_layers: 1,
        },
        bevy::render::render_resource::TextureDimension::D2,
        data,
        bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
        bevy::asset::RenderAssetUsages::default(),
    )
}

#[cfg(test)]
mod tests {
    use super::test_pattern_image;

    #[test]
    fn test_usage() {
        assert_eq!(
            test_pattern_image().size(),
            bevy::math::UVec2::new(160, 100)
        );
    }
}
