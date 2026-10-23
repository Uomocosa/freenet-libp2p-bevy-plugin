use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::starboard;

pub fn fit_board(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut view: ResMut<starboard::render::CanvasView>,
    board: Single<(&mut Sprite, &mut Transform), With<starboard::render::BoardSprite>>,
    mut borders: Query<
        &mut Sprite,
        (
            With<starboard::render::BoardBorderSprite>,
            Without<starboard::render::BoardSprite>,
        ),
    >,
) {
    let Ok(window) = windows.single() else {
        return;
    };
    let next = starboard::render::CanvasView::fit(window.size());
    if (view.scale - next.scale).abs() > f32::EPSILON
        || (view.offset.x - next.offset.x).abs() > f32::EPSILON
        || (view.offset.y - next.offset.y).abs() > f32::EPSILON
    {
        *view = next;
    }
    let (mut sprite, mut transform) = board.into_inner();
    let side =
        f32::from(u16::try_from(starboard::canvas::CANVAS_SIDE).unwrap_or(1024)) * next.scale;
    sprite.custom_size = Some(Vec2::splat(side));
    transform.translation = Vec3::ZERO;
    for mut border in &mut borders {
        border.custom_size = Some(Vec2::splat(
            2.0_f32.mul_add(starboard::render::BOARD_BORDER_PX, side),
        ));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;

    use super::fit_board;
    use crate::starboard;

    fn preview_build(app: &mut App) {
        app.init_resource::<starboard::render::CanvasView>()
            .init_resource::<starboard::input::OwnBrush>()
            .add_systems(Startup, starboard::render::bevy_systems::spawn_board)
            .add_systems(Update, fit_board);
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_resize(world: &mut World) {
        let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
        let Ok(mut window) = windows.single_mut(world) else {
            return;
        };
        window.resolution.set(640.0, 900.0);
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
            name: String::from("starboard_fit_board"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("wide window"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("tall window"),
                    apply: preview_resize,
                },
            ],
            timeline: None,
        }
    }

    #[test]
    fn fit_board_ui_png_preview() {
        scene()
            .run(&config(), env!("CARGO_PKG_NAME"))
            .expect("preview");
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<starboard::render::CanvasView>();
        app.world_mut().spawn((
            PrimaryWindow,
            Window {
                resolution: (1600, 900).into(),
                ..default()
            },
        ));
        app.world_mut().spawn((
            starboard::render::BoardSprite,
            Sprite::default(),
            Transform::default(),
        ));
        app.add_systems(Update, fit_board);
        app.update();

        let view = *app.world().resource::<starboard::render::CanvasView>();
        assert!((view.scale - 900.0 / 1024.0).abs() < 0.000_1);
        assert!((view.offset.x - 350.0).abs() < 0.000_1);

        let mut boards = app
            .world_mut()
            .query_filtered::<&Sprite, With<starboard::render::BoardSprite>>();
        let sprite = boards.single(app.world()).expect("one board sprite");
        assert_eq!(sprite.custom_size, Some(Vec2::splat(900.0)));
    }
}
