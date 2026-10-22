use bevy::prelude::*;

use crate::starboard;

pub fn repaint_own_brush_outline(
    view: Res<starboard::render::CanvasView>,
    brush: Res<starboard::input::OwnBrush>,
    mut images: ResMut<Assets<Image>>,
    mut outlines: Query<
        (&starboard::render::Owner, &mut Sprite),
        With<starboard::render::CursorOutline>,
    >,
) {
    if !view.is_changed() && !brush.is_changed() {
        return;
    }
    let view = view.into_inner();
    let brush = brush.into_inner();
    let Some((_, mut sprite)) = outlines.iter_mut().find(|(owner, _)| owner.is_me()) else {
        return;
    };
    let next = starboard::render::outline_brush(
        brush.shape,
        starboard::canvas::brush_radius(brush.size_index),
        view.scale,
    );
    sprite.custom_size = Some(Vec2::splat(next.screen_side));
    if let Some(mut image) = images.get_mut(&sprite.image) {
        starboard::render::replace_pixels(&mut image, next.image);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::repaint_own_brush_outline;
    use crate::starboard;

    fn ready_status() -> discovery::Status {
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from(0),
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
            }),
            ..discovery::Status::default()
        }
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(ready_status());
        app.init_resource::<starboard::render::CanvasView>()
            .init_resource::<starboard::input::OwnBrush>()
            .init_resource::<ButtonInput<MouseButton>>()
            .insert_resource(starboard::input::CursorCell(Some(
                starboard::canvas::CellPos { x: 512, y: 512 },
            )))
            .add_systems(Startup, starboard::render::bevy_systems::spawn_board)
            .add_systems(
                Update,
                (
                    starboard::render::bevy_systems::fit_board,
                    starboard::render::bevy_systems::follow_brush,
                    repaint_own_brush_outline,
                )
                    .chain(),
            );
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_bigger_brush(world: &mut World) {
        world
            .resource_mut::<starboard::input::OwnBrush>()
            .size_index = 4;
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
            name: String::from("starboard_repaint_own_brush_outline"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("default brush"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("bigger brush"),
                    apply: preview_bigger_brush,
                },
            ],
            timeline: None,
        }
    }

    #[test]
    fn repaint_own_brush_outline_ui_png_preview() {
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
        app.insert_resource(starboard::render::CanvasView::fit(Vec2::splat(1024.0)));
        app.init_resource::<starboard::input::OwnBrush>();
        let image = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(starboard::render::square_image(1, vec![0_u8; 4]));
        app.world_mut().spawn((
            starboard::render::CursorOutline,
            starboard::render::Owner::Me,
            Sprite::from_image(image),
        ));
        app.add_systems(Update, repaint_own_brush_outline);
        app.update();

        let mut outlines = app
            .world_mut()
            .query_filtered::<&Sprite, With<starboard::render::CursorOutline>>();
        let sprite = outlines.single(app.world()).expect("one outline");
        assert!(sprite.custom_size.is_some_and(|size| size.x > 1.0));
    }
}
