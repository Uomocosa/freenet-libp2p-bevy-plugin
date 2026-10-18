use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::room_sync;

use crate::starboard;

pub struct StarboardPlugin;

impl Plugin for StarboardPlugin {
    fn build(&self, app: &mut App) {
        use starboard::{input, render};
        app.init_resource::<starboard::canvas::Canvas>()
            .init_resource::<render::CanvasView>()
            .init_resource::<input::OwnBrush>()
            .init_resource::<input::LatestPaintedCell>()
            .init_resource::<room_sync::LamportClock>()
            .add_message::<input::PendingLocalDraw>()
            .add_systems(Startup, render::bevy_systems::spawn_board)
            .add_systems(
                Update,
                (
                    input::bevy_systems::scroll_brush_size,
                    render::bevy_systems::fit_board,
                    input::bevy_systems::track_pointer,
                    input::bevy_systems::apply_local_draws,
                    render::bevy_systems::upload_dirty_tiles,
                    render::bevy_systems::follow_brush,
                )
                    .chain(),
            );
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;
    use freenet_libp2p_bevy_plugin::room_sync::{OpMeta, RoomState};

    use super::StarboardPlugin;
    use crate::starboard;

    fn preview_build(app: &mut App) {
        app.add_plugins(StarboardPlugin);
    }

    const fn preview_noop(_world: &mut World) {}

    fn paint_strokes(world: &mut World) {
        let brush = *world.resource::<starboard::input::OwnBrush>();
        let mut canvas = world.resource_mut::<starboard::canvas::Canvas>();
        for index in 0_u16..28 {
            let x = 96_u16.saturating_add(index.saturating_mul(24));
            let y = 300_u16.saturating_add(index.saturating_mul(7));
            let op = starboard::input::draw_op(
                brush,
                starboard::canvas::DrawAction::Paint(brush.color),
                starboard::canvas::CellPos { x, y },
                starboard::canvas::CellPos {
                    x: x.saturating_add(40),
                    y: y.saturating_add(60),
                },
            );
            canvas.apply(
                OpMeta {
                    lamport: u32::from(index).saturating_add(1),
                    author: 1,
                },
                &op,
            );
        }
    }

    fn show_cursor(world: &mut World) {
        let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
        let Ok(mut window) = windows.single_mut(world) else {
            return;
        };
        window.set_cursor_position(Some(Vec2::new(560.0, 300.0)));
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
            name: String::from("starboard_board"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("empty board"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("painted board"),
                    apply: paint_strokes,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("brush over the board"),
                    apply: show_cursor,
                },
            ],
            timeline: None,
        }
    }

    #[test]
    fn board_ui_png_preview() {
        lele_bevy_preview::run(&scene(), &config(), env!("CARGO_PKG_NAME")).expect("preview");
    }

    #[test]
    fn board_ui_scene_preview() {
        lele_bevy_preview::run(&scene(), &config(), env!("CARGO_PKG_NAME")).expect("preview");
    }
}
