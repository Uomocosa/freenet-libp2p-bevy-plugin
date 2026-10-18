use bevy::prelude::*;

use crate::starboard;

pub fn upload_dirty_tiles(
    mut canvas: ResMut<starboard::canvas::Canvas>,
    board: Single<&Sprite, With<starboard::render::BoardSprite>>,
    mut images: ResMut<Assets<Image>>,
) {
    if canvas.dirty_tiles.is_empty() {
        return;
    }
    let board = board.into_inner();
    let Some(mut image) = images.get_mut(&board.image) else {
        return;
    };
    let Some(data) = image.data.as_mut() else {
        return;
    };
    let _ = starboard::render::repaint_dirty_tiles(&mut canvas, data);
}

#[cfg(test)]
mod tests {
    use bevy::asset::RenderAssetUsages;
    use bevy::prelude::*;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

    use super::upload_dirty_tiles;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        app.init_resource::<starboard::canvas::Canvas>();
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::new_fill(
                Extent3d {
                    width: 1024,
                    height: 1024,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                &starboard::render::CANVAS_BACKGROUND,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            ));
        app.world_mut().spawn((
            starboard::render::BoardSprite,
            Sprite::from_image(image.clone()),
        ));

        let color = starboard::identity::PaletteColor(6);
        let mut canvas = app.world_mut().resource_mut::<starboard::canvas::Canvas>();
        assert!(starboard::canvas::write_cell(
            &mut canvas,
            starboard::canvas::CellPos { x: 5, y: 7 },
            starboard::canvas::Cell {
                lamport: 1,
                author: 2,
                color: Some(color),
            }
        ));
        drop(canvas);

        app.add_systems(Update, upload_dirty_tiles);
        app.update();

        let images = app.world().resource::<Assets<Image>>();
        let image = images.get(&image).expect("board image");
        let data = image.data.as_ref().expect("cpu pixels");
        let start = (7 * 1024 + 5) * 4;
        assert_eq!(
            &data[start..start + 4],
            &starboard::render::palette_rgba(color)
        );
    }
}
