use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::{discovery, room_sync};

use freenet_libp2p_bevy_plugin::ui;

use crate::starboard;

pub struct StarboardPlugin;

impl Plugin for StarboardPlugin {
    fn build(&self, app: &mut App) {
        use starboard::{input, render, room};
        let in_room =
            |state: Option<Res<DiscoveryState>>| state.as_deref().is_some_and(room::is_in_room);
        app.init_resource::<starboard::canvas::Canvas>()
            .init_resource::<render::CanvasView>()
            .init_resource::<input::OwnBrush>()
            .init_resource::<input::LatestPaintedCell>()
            .init_resource::<input::PendingPressState>()
            .init_resource::<input::CursorCell>()
            .init_resource::<room_sync::LamportClock>()
            .init_resource::<room_sync::PendingOps<starboard::canvas::Canvas>>()
            .init_resource::<room::RoomBrushes>()
            .init_resource::<room::RemoteCursors>()
            .init_resource::<ui::roster::MemberIcons>()
            .insert_resource(room::BrushRefreshTimer(Timer::from_seconds(
                room::BRUSH_REFRESH_SECONDS,
                TimerMode::Repeating,
            )))
            .add_message::<input::PendingLocalDraw>()
            .add_message::<discovery::Event>()
            .add_systems(Startup, render::bevy_systems::spawn_board)
            .add_systems(
                Update,
                (
                    room::bevy_systems::reset_canvas,
                    room::bevy_systems::refresh_room_brushes
                        .in_set(starboard::app::WallClockSystem::RoomBrushes),
                    room::bevy_systems::refresh_member_icons,
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
    use freenet_libp2p_bevy_plugin::room_sync::{OpMeta, traits::ReplicatedState};
    use freenet_libp2p_bevy_plugin::time;
    use freenet_libp2p_bevy_plugin::{discovery, identity, testing};

    use rstest::rstest;

    use crate::starboard;
    use crate::starboard::app::test_harness;
    use discovery::state_machine::DiscoveryState;

    fn ready_status(me: &identity::PeerId) -> DiscoveryState {
        testing::DiscoveryStateBuilder::new(me.clone())
            .in_room(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: time::UnixTime::EPOCH,
                phase: discovery::JoinPhase::Ready,
                members: discovery::Members::new(),
                how_entered: discovery::HowRoomWasEntered::Created,
            })
            .build()
    }

    const CIRCLE_RED: starboard::identity::DailyBrush = starboard::identity::DailyBrush {
        shape: starboard::identity::Shape::Circle,
        color: starboard::identity::PaletteColor(0),
    };
    const SQUARE_BLUE: starboard::identity::DailyBrush = starboard::identity::DailyBrush {
        shape: starboard::identity::Shape::Square,
        color: starboard::identity::PaletteColor(6),
    };
    const STAR_YELLOW: starboard::identity::DailyBrush = starboard::identity::DailyBrush {
        shape: starboard::identity::Shape::Star,
        color: starboard::identity::PaletteColor(11),
    };

    /// The brush a board preview paints with, stored by its case instead of read from `OwnBrush`.
    #[derive(Resource, Clone, Copy)]
    struct CaseBrush(starboard::identity::DailyBrush);

    fn preview_build(app: &mut App, brush: starboard::identity::DailyBrush) {
        let me = identity::PeerId::from("preview-me");
        app.insert_resource(ready_status(&me));
        app.insert_resource(starboard::room::RoomBrushes(
            std::collections::BTreeMap::from([(me, brush)]),
        ));
        app.insert_resource(starboard::input::OwnBrush {
            shape: brush.shape,
            color: brush.color,
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
        });
        app.insert_resource(CaseBrush(brush));
        app.add_plugins(starboard::app::StarboardTestPlugin::default());
    }

    fn build_circle_red(app: &mut App) {
        preview_build(app, CIRCLE_RED);
    }

    fn build_square_blue(app: &mut App) {
        preview_build(app, SQUARE_BLUE);
    }

    fn build_star_yellow(app: &mut App) {
        preview_build(app, STAR_YELLOW);
    }

    const fn preview_noop(_world: &mut World) {}

    /// One curved stroke, as segments between these offsets from its origin.
    const CURVE: [(u16, i16); 9] = [
        (0, 0),
        (12, 14),
        (24, 24),
        (36, 28),
        (48, 24),
        (60, 14),
        (72, 0),
        (84, -14),
        (96, -24),
    ];

    fn brush_of(brush: starboard::identity::DailyBrush) -> starboard::input::OwnBrush {
        starboard::input::OwnBrush {
            shape: brush.shape,
            color: brush.color,
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
        }
    }

    fn paint_op(
        canvas: &mut starboard::canvas::Canvas,
        brush: starboard::input::OwnBrush,
        stroke: starboard::canvas::Stroke,
        lamport: &mut u32,
    ) {
        *lamport = lamport.saturating_add(1);
        let op = starboard::input::draw_op(
            brush,
            starboard::canvas::DrawAction::Paint(brush.color),
            stroke,
        );
        canvas.apply(
            OpMeta {
                lamport: *lamport,
                author: 1,
            },
            &op,
        );
    }

    fn paint_curve(
        canvas: &mut starboard::canvas::Canvas,
        brush: starboard::input::OwnBrush,
        origin: starboard::canvas::CellPos,
        lamport: &mut u32,
    ) {
        let points: Vec<starboard::canvas::CellPos> = CURVE
            .iter()
            .map(|&(dx, dy)| starboard::canvas::CellPos {
                x: origin.x.saturating_add(dx),
                y: origin.y.saturating_add_signed(dy),
            })
            .collect();
        for (from, to) in points.iter().zip(points.iter().skip(1)) {
            let stroke = starboard::canvas::Stroke {
                from: *from,
                to: *to,
            };
            paint_op(canvas, brush, stroke, lamport);
        }
    }

    /// Three curved strokes in the case brush, then three single dots.
    fn paint_strokes(world: &mut World) {
        let brush = brush_of(world.resource::<CaseBrush>().0);
        let mut canvas = world.resource_mut::<starboard::canvas::Canvas>();
        let mut lamport = 0_u32;
        for y in [150_u16, 400, 650] {
            let origin = starboard::canvas::CellPos { x: 120, y };
            paint_curve(&mut canvas, brush, origin, &mut lamport);
        }
        for (x, y) in [(820_u16, 200_u16), (860, 260), (820, 330)] {
            let dot = starboard::canvas::Stroke {
                from: starboard::canvas::CellPos { x, y },
                to: starboard::canvas::CellPos { x, y },
            };
            paint_op(&mut canvas, brush, dot, &mut lamport);
        }
    }

    /// One curved stroke with the star brush, painted over the board in later states.
    fn paint_star_stroke(world: &mut World) {
        let brush = brush_of(STAR_YELLOW);
        let mut canvas = world.resource_mut::<starboard::canvas::Canvas>();
        let mut lamport = 1000_u32;
        let origin = starboard::canvas::CellPos { x: 300, y: 520 };
        paint_curve(&mut canvas, brush, origin, &mut lamport);
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

    fn board_scene(name: &str, build: fn(&mut App)) -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from(name),
            kind: lele_bevy_preview::scene::Kind::App,
            build,
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
                lele_bevy_preview::scene::State {
                    label: String::from("star stroke"),
                    apply: paint_star_stroke,
                },
            ],
            timeline: None,
        }
    }

    #[rstest]
    #[case::circle_red("starboard_board_circle_red", build_circle_red)]
    #[case::square_blue("starboard_board_square_blue", build_square_blue)]
    #[case::star_yellow("starboard_board_star_yellow", build_star_yellow)]
    fn board_ui_png_preview(#[case] name: &str, #[case] build: fn(&mut App)) {
        board_scene(name, build).run(&config()).expect("preview");
    }

    #[rstest]
    #[case::circle_red("starboard_board_circle_red", build_circle_red)]
    #[case::square_blue("starboard_board_square_blue", build_square_blue)]
    #[case::star_yellow("starboard_board_star_yellow", build_star_yellow)]
    fn board_ui_scene_preview(#[case] name: &str, #[case] build: fn(&mut App)) {
        board_scene(name, build).run(&config()).expect("preview");
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

    #[tokio::test(flavor = "multi_thread")]
    async fn two_apps_painting_the_same_cells_right_after_joining_converge() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("room_sync=debug")),
            )
            .with_ansi(false)
            .try_init();
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

        for op in scripted_ops() {
            for app in &mut apps {
                app.world_mut()
                    .write_message(starboard::input::PendingLocalDraw(op));
                app.update();
            }
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

    #[tokio::test(flavor = "multi_thread")]
    async fn a_joiner_learns_the_creators_room_subscription_and_both_converge() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("room_sync=debug")),
            )
            .with_ansi(false)
            .try_init();
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-ae");
        let mut apps = vec![
            test_harness::node_with(&game, None),
            test_harness::node_with(&game, None),
        ];
        let peers: Vec<identity::Peer> = apps.iter_mut().map(test_harness::wait_ready).collect();

        test_harness::join(&mut apps[0], &room, Vec::new());
        for _ in 0..20 {
            apps[0].update();
            tokio::time::sleep(test_harness::SETTLE).await;
        }

        test_harness::dial(&mut apps[1], &peers[0]);
        test_harness::join(&mut apps[1], &room, vec![peers[0].id.clone()]);

        for op in scripted_ops() {
            apps[1]
                .world_mut()
                .write_message(starboard::input::PendingLocalDraw(op));
            apps[1].update();
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
