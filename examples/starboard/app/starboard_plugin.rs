use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{discovery, room_sync};

use crate::starboard;

pub struct StarboardPlugin;

impl Plugin for StarboardPlugin {
    fn build(&self, app: &mut App) {
        use starboard::{input, render, room, ui};
        let in_room = |status: Res<discovery::Status>| room::is_in_room(&status);
        app.init_resource::<starboard::canvas::Canvas>()
            .init_resource::<render::CanvasView>()
            .init_resource::<input::OwnBrush>()
            .init_resource::<input::LatestPaintedCell>()
            .init_resource::<input::CursorCell>()
            .init_resource::<room_sync::LamportClock>()
            .init_resource::<room_sync::PendingOps<starboard::canvas::Canvas>>()
            .init_resource::<room::RoomBrushes>()
            .init_resource::<room::RemoteCursors>()
            .insert_resource(room::BrushRefreshTimer(Timer::from_seconds(
                room::BRUSH_REFRESH_SECONDS,
                TimerMode::Repeating,
            )))
            .add_message::<input::PendingLocalDraw>()
            .add_message::<discovery::Event>()
            .add_systems(
                Startup,
                (
                    render::bevy_systems::spawn_board,
                    ui::bevy_systems::spawn_member_panel,
                ),
            )
            .add_systems(
                Update,
                (
                    room::bevy_systems::reset_canvas,
                    room::bevy_systems::refresh_room_brushes,
                    ui::bevy_systems::rebuild_member_list,
                    input::bevy_systems::track_cursor,
                    render::bevy_systems::sync_board_visibility,
                    render::bevy_systems::fit_board,
                    input::bevy_systems::scroll_brush_size.run_if(in_room),
                    input::bevy_systems::track_pointer.run_if(in_room),
                    input::bevy_systems::send_local_draws.run_if(in_room),
                    room::bevy_systems::publish_cursor.run_if(in_room),
                    room::bevy_systems::collect_remote_cursors,
                    render::bevy_systems::upload_dirty_tiles,
                    render::bevy_systems::follow_brush,
                    render::bevy_systems::repaint_own_brush_outline,
                    render::bevy_systems::draw_remote_cursors,
                    render::bevy_systems::draw_cursor_labels,
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
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::StarboardPlugin;
    use crate::starboard;
    use crate::starboard::app::test_harness;

    fn ready_status() -> discovery::Status {
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(0),
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
            }),
            ..discovery::Status::default()
        }
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(ready_status());
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
                starboard::canvas::Stroke {
                    from: starboard::canvas::CellPos { x, y },
                    to: starboard::canvas::CellPos {
                        x: x.saturating_add(40),
                        y: y.saturating_add(60),
                    },
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

    const CONVERGE_BUDGET: std::time::Duration = std::time::Duration::from_secs(10);

    fn scripted_ops() -> Vec<starboard::canvas::DrawOp> {
        let brush = starboard::input::OwnBrush::default();
        (0_u16..6)
            .map(|index| {
                let x = 200_u16.saturating_add(index.saturating_mul(60));
                let y = 300_u16.saturating_add(index.saturating_mul(30));
                starboard::input::draw_op(
                    brush,
                    starboard::canvas::DrawAction::Paint(brush.color),
                    starboard::canvas::Stroke {
                        from: starboard::canvas::CellPos { x, y },
                        to: starboard::canvas::CellPos {
                            x: x.saturating_add(80),
                            y: y.saturating_add(40),
                        },
                    },
                )
            })
            .collect()
    }

    fn canvas_hash(app: &App) -> u64 {
        starboard::canvas::hash_canvas(app.world().resource::<starboard::canvas::Canvas>())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn two_apps_converge_on_the_painted_canvas() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-ae");
        let mut apps = vec![
            test_harness::node_with(&game, None),
            test_harness::node_with(&game, None),
        ];
        let peers: Vec<identity::Peer> = apps.iter_mut().map(test_harness::wait_ready).collect();

        test_harness::dial(&mut apps[1], &peers[0]);

        for (index, app) in apps.iter_mut().enumerate() {
            let others = peers
                .iter()
                .enumerate()
                .filter(|(other, _)| *other != index)
                .map(|(_, peer)| peer.id.clone())
                .collect();
            test_harness::join(app, &room, others);
        }
        for _ in 0..50 {
            for app in &mut apps {
                app.update();
            }
            tokio::time::sleep(test_harness::SETTLE).await;
        }

        for op in scripted_ops() {
            apps[0]
                .world_mut()
                .write_message(starboard::input::PendingLocalDraw(op));
            apps[0].update();
        }

        let started = std::time::Instant::now();
        loop {
            for app in &mut apps {
                app.update();
            }
            let hashes: Vec<u64> = apps.iter().map(canvas_hash).collect();
            if hashes.first() == hashes.last() && hashes.first().is_some_and(|hash| *hash != 0) {
                return;
            }
            assert!(
                started.elapsed() <= CONVERGE_BUDGET,
                "canvases did not converge: {hashes:?}"
            );
            tokio::time::sleep(test_harness::SETTLE).await;
        }
    }
}
