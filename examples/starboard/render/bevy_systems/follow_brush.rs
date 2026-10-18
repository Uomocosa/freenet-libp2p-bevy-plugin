use bevy::prelude::*;
use bevy::render::render_resource::Extent3d;
use bevy::window::PrimaryWindow;
use freenet_libp2p_bevy_plugin::discovery;

use crate::starboard;

pub fn follow_brush(
    windows: Query<&Window, With<PrimaryWindow>>,
    status: Res<discovery::Status>,
    cursor: Res<starboard::input::CursorCell>,
    view: Res<starboard::render::CanvasView>,
    brush: Res<starboard::input::OwnBrush>,
    mut images: ResMut<Assets<Image>>,
    outline: Single<
        (&mut Sprite, &mut Transform, &mut Visibility),
        With<starboard::render::BrushOutlineSprite>,
    >,
) {
    let status = status.into_inner();
    let is_in_room = starboard::room::is_in_room(status);
    let is_brush_changed = brush.is_changed();
    let cursor = cursor.into_inner();
    let view = view.into_inner();
    let brush = brush.into_inner();
    let (mut sprite, mut transform, mut visibility) = outline.into_inner();
    if !is_in_room {
        *visibility = Visibility::Hidden;
        return;
    }
    if is_brush_changed {
        let next = starboard::render::brush_outline(
            brush.shape,
            starboard::canvas::brush_radius(brush.size_index),
            brush.color,
        );
        if let Some(mut image) = images.get_mut(&sprite.image) {
            image.resize(Extent3d {
                width: next.side,
                height: next.side,
                depth_or_array_layers: 1,
            });
            image.data = Some(next.pixels);
        }
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cell) = **cursor else {
        *visibility = Visibility::Hidden;
        return;
    };
    let radius = starboard::canvas::brush_radius(brush.size_index);
    let side = f32::from(radius.saturating_mul(2).saturating_add(1));
    sprite.custom_size = Some(Vec2::splat(side * view.scale));
    transform.translation = starboard::render::cell_center(cell, view, window.size());
    *visibility = Visibility::Visible;
}

#[cfg(test)]
mod tests {
    use bevy::asset::RenderAssetUsages;
    use bevy::prelude::*;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    use bevy::window::PrimaryWindow;
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::follow_brush;
    use crate::starboard;

    fn ready_status() -> discovery::Status {
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(0),
                phase: discovery::RoomPhase::Ready,
                members: discovery::Members::new(),
            }),
            ..discovery::Status::default()
        }
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(ready_status());
        app.init_resource::<starboard::render::CanvasView>()
            .init_resource::<starboard::input::OwnBrush>()
            .init_resource::<starboard::input::CursorCell>()
            .add_systems(Startup, starboard::render::bevy_systems::spawn_board)
            .add_systems(
                Update,
                (starboard::render::bevy_systems::fit_board, follow_brush),
            );
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_cursor(world: &mut World) {
        **world.resource_mut::<starboard::input::CursorCell>() =
            Some(starboard::canvas::CellPos { x: 300, y: 200 });
    }

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene() -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("starboard_follow_brush"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("cursor outside"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("cursor on the board"),
                    apply: preview_cursor,
                },
            ],
            timeline: None,
        }
    }

    #[test]
    fn follow_brush_ui_png_preview() {
        lele_bevy_preview::run(&scene(), &config(), env!("CARGO_PKG_NAME")).expect("preview");
    }

    fn app_with_outline() -> App {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        app.insert_resource(starboard::render::CanvasView::fit(Vec2::splat(1024.0)));
        app.insert_resource(ready_status());
        let brush = starboard::input::OwnBrush::default();
        app.insert_resource(brush);
        let starboard::render::BrushOutline { side, pixels } = starboard::render::brush_outline(
            brush.shape,
            starboard::canvas::brush_radius(brush.size_index),
            brush.color,
        );
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(Image::new(
                Extent3d {
                    width: side,
                    height: side,
                    depth_or_array_layers: 1,
                },
                TextureDimension::D2,
                pixels,
                TextureFormat::Rgba8UnormSrgb,
                RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
            ));
        app.world_mut().spawn((
            PrimaryWindow,
            Window {
                resolution: (1024, 1024).into(),
                ..default()
            },
        ));
        app.world_mut().spawn((
            starboard::render::BrushOutlineSprite,
            Sprite::from_image(image),
            Transform::default(),
            Visibility::Hidden,
        ));
        app.init_resource::<starboard::input::CursorCell>();
        app.add_systems(Update, follow_brush);
        app
    }

    fn set_cursor(app: &mut App, cell: Option<starboard::canvas::CellPos>) {
        **app
            .world_mut()
            .resource_mut::<starboard::input::CursorCell>() = cell;
    }

    #[test]
    fn test_usage() {
        let mut app = app_with_outline();
        set_cursor(
            &mut app,
            Some(starboard::canvas::CellPos { x: 300, y: 200 }),
        );
        app.update();

        let mut outlines = app.world_mut().query_filtered::<
            (&Transform, &Visibility, &Sprite),
            With<starboard::render::BrushOutlineSprite>,
        >();
        let (transform, visibility, sprite) =
            outlines.single(app.world()).expect("one outline sprite");
        assert_eq!(*visibility, Visibility::Visible);
        assert!((transform.translation.x - -211.5).abs() < 0.000_1);
        assert!((transform.translation.y - 311.5).abs() < 0.000_1);
        assert_eq!(sprite.custom_size, Some(Vec2::splat(33.0)));

        set_cursor(&mut app, None);
        app.update();
        let mut outlines = app
            .world_mut()
            .query_filtered::<&Visibility, With<starboard::render::BrushOutlineSprite>>();
        assert_eq!(
            *outlines.single(app.world()).expect("one outline sprite"),
            Visibility::Hidden
        );
    }
}
