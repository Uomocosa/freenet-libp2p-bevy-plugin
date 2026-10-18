use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::starboard;

pub fn spawn_board(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    brush: Res<starboard::input::OwnBrush>,
) {
    let brush = brush.into_inner();
    let board = images.add(canvas_image());
    commands.spawn((
        starboard::render::BoardSprite,
        Sprite::from_image(board),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
    let outline = images.add(outline_image(*brush));
    commands.spawn((
        starboard::render::BrushOutlineSprite,
        Sprite::from_image(outline),
        Visibility::Hidden,
    ));
}

// needed helper: the 1024x1024 RGBA canvas kept on the CPU and re-uploaded when a tile changes
fn canvas_image() -> Image {
    let side = u32::try_from(starboard::canvas::CANVAS_SIDE).unwrap_or(1024);
    Image::new_fill(
        Extent3d {
            width: side,
            height: side,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        &starboard::render::CANVAS_BACKGROUND,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    )
}

// needed helper: the cursor outline texture for the brush that is current when the board spawns
fn outline_image(brush: starboard::input::OwnBrush) -> Image {
    let starboard::render::BrushOutline { side, pixels } = starboard::render::brush_outline(
        brush.shape,
        starboard::canvas::brush_radius(brush.size_index),
        brush.color,
    );
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
    use bevy::prelude::*;

    use super::spawn_board;
    use crate::starboard;

    fn preview_build(app: &mut App) {
        app.init_resource::<starboard::input::OwnBrush>()
            .init_resource::<starboard::render::CanvasView>()
            .add_systems(Startup, spawn_board)
            .add_systems(Update, starboard::render::bevy_systems::fit_board);
    }

    const fn preview_noop(_world: &mut World) {}

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene() -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("starboard_spawn_board"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("empty board"),
                apply: preview_noop,
            }],
            timeline: None,
        }
    }

    #[test]
    fn spawn_board_ui_png_preview() {
        lele_bevy_preview::run(&scene(), &config(), env!("CARGO_PKG_NAME")).expect("preview");
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        app.insert_resource(starboard::input::OwnBrush::default());
        app.add_systems(Startup, spawn_board);
        app.update();

        let mut boards = app
            .world_mut()
            .query_filtered::<&Sprite, With<starboard::render::BoardSprite>>();
        let board = boards
            .single(app.world())
            .expect("one board sprite")
            .image
            .clone();
        let images = app.world().resource::<Assets<Image>>();
        let image = images.get(&board).expect("board image");
        assert_eq!(image.texture_descriptor.size.width, 1024);
        assert_eq!(image.texture_descriptor.size.height, 1024);

        let mut outlines = app
            .world_mut()
            .query_filtered::<&Visibility, With<starboard::render::BrushOutlineSprite>>();
        assert_eq!(
            *outlines.single(app.world()).expect("one outline sprite"),
            Visibility::Hidden
        );
    }
}
