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
            .init_resource::<input::CursorPoint>()
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

impl StarboardPlugin {
    pub(crate) fn spawn_camera(mut commands: Commands) {
        commands.spawn(Camera2d);
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;
    use freenet_libp2p_bevy_plugin::room_sync::{OpMeta, PendingOps, traits::ReplicatedState};
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
        points: Vec<starboard::canvas::SubCellPos>,
        stroke_rank: u32,
        author: u16,
    ) {
        let op = starboard::input::draw_op(
            brush,
            starboard::canvas::DrawAction::Paint(brush.color),
            points,
            stroke_rank,
        );
        canvas.apply(
            OpMeta {
                lamport: stroke_rank,
                author,
            },
            &op,
        );
    }

    fn paint_points(
        canvas: &mut starboard::canvas::Canvas,
        brush: starboard::input::OwnBrush,
        points: &[starboard::canvas::CellPos],
        stroke_rank: u32,
        author: u16,
    ) {
        for (from, to) in points.iter().zip(points.iter().skip(1)) {
            let segment = vec![
                starboard::canvas::cell_centre(*from),
                starboard::canvas::cell_centre(*to),
            ];
            paint_op(canvas, brush, segment, stroke_rank, author);
        }
    }

    fn paint_curve(
        canvas: &mut starboard::canvas::Canvas,
        brush: starboard::input::OwnBrush,
        origin: starboard::canvas::CellPos,
        stroke_rank: u32,
    ) {
        let points: Vec<starboard::canvas::CellPos> = CURVE
            .iter()
            .map(|&(dx, dy)| starboard::canvas::CellPos {
                x: origin.x.saturating_add(dx),
                y: origin.y.saturating_add_signed(dy),
            })
            .collect();
        paint_points(canvas, brush, &points, stroke_rank, 1);
    }

    /// Three curved strokes in the case brush, then three single dots. Each stroke starts later.
    fn paint_strokes(world: &mut World) {
        let brush = brush_of(world.resource::<CaseBrush>().0);
        let mut canvas = world.resource_mut::<starboard::canvas::Canvas>();
        let mut stroke_rank = 0_u32;
        for y in [150_u16, 400, 650] {
            stroke_rank = stroke_rank.saturating_add(1);
            let origin = starboard::canvas::CellPos { x: 120, y };
            paint_curve(&mut canvas, brush, origin, stroke_rank);
        }
        for (x, y) in [(820_u16, 200_u16), (860, 260), (820, 330)] {
            stroke_rank = stroke_rank.saturating_add(1);
            let cell = starboard::canvas::cell_centre(starboard::canvas::CellPos { x, y });
            let dot = vec![cell, cell];
            paint_op(&mut canvas, brush, dot, stroke_rank, 1);
        }
    }

    /// One curved stroke with the star brush, painted over the board in later states.
    fn paint_star_stroke(world: &mut World) {
        let brush = brush_of(STAR_YELLOW);
        let mut canvas = world.resource_mut::<starboard::canvas::Canvas>();
        let origin = starboard::canvas::CellPos { x: 300, y: 520 };
        paint_curve(&mut canvas, brush, origin, 1001);
    }

    /// Zig-zag heights in units of `CROSSING_UNIT_PX`, one per point. The sign changes at
    /// i = 3, 9 and 15, so two strokes mirrored about the middle row cross exactly three times.
    const CROSSING_HEIGHTS: [i16; 19] = [
        -6, -4, -2, 0, 2, 4, 6, 4, 2, 0, -2, -4, -6, -4, -2, 0, 2, 4, 6,
    ];
    const CROSSING_UNIT_PX: i16 = 20;
    const CROSSING_STEP_PX: u16 = 40;
    const CROSSING_ROW: i16 = 450;

    fn crossing_points(sign: i16) -> Vec<starboard::canvas::CellPos> {
        CROSSING_HEIGHTS
            .iter()
            .enumerate()
            .map(|(index, height)| starboard::canvas::CellPos {
                x: 120_u16.saturating_add(
                    u16::try_from(index)
                        .unwrap_or(0)
                        .saturating_mul(CROSSING_STEP_PX),
                ),
                y: CROSSING_ROW
                    .saturating_add(
                        sign.saturating_mul(*height)
                            .saturating_mul(CROSSING_UNIT_PX),
                    )
                    .unsigned_abs(),
            })
            .collect()
    }

    /// Two differently coloured strokes that cross three times. The later-started stroke (higher
    /// rank) is painted first; the earlier one's segments are applied last and must still lose
    /// at every crossing.
    fn paint_crossing_strokes(world: &mut World) {
        let earlier = starboard::input::OwnBrush {
            shape: starboard::identity::Shape::Circle,
            color: starboard::identity::PaletteColor(0),
            size_index: starboard::canvas::DEFAULT_SIZE_INDEX,
        };
        let later = starboard::input::OwnBrush {
            color: starboard::identity::PaletteColor(6),
            ..earlier
        };
        let mut canvas = world.resource_mut::<starboard::canvas::Canvas>();
        paint_points(&mut canvas, later, &crossing_points(-1), 2, 2);
        paint_points(&mut canvas, earlier, &crossing_points(1), 1, 1);
    }

    /// Brush size for the edge close-up: big enough that the swept edge spans many cells.
    const EDGE_CLOSE_UP_SIZE_INDEX: u8 = 7;

    /// A diagonal stroke off the cell grid and a curved stroke, both with a big brush of the case
    /// shape, so the swept edge can be inspected on the board.
    fn paint_edge_close_up(world: &mut World) {
        let case = brush_of(world.resource::<CaseBrush>().0);
        let brush = starboard::input::OwnBrush {
            size_index: EDGE_CLOSE_UP_SIZE_INDEX,
            ..case
        };
        let mut canvas = world.resource_mut::<starboard::canvas::Canvas>();
        let diagonal = vec![
            starboard::canvas::SubCellPos {
                x: 600 * 8 + 3,
                y: 650 * 8 + 5,
            },
            starboard::canvas::SubCellPos {
                x: 860 * 8 + 2,
                y: 830 * 8 + 1,
            },
        ];
        paint_op(&mut canvas, brush, diagonal, 1100, 1);
        paint_curve(
            &mut canvas,
            brush,
            starboard::canvas::CellPos { x: 150, y: 760 },
            1101,
        );
    }

    const S_CURVE_START: u16 = 120;
    const S_CURVE_LENGTH: u16 = 720;
    const S_CURVE_AMPLITUDE: f32 = 90.0;
    const FAST_STEP_CELLS: u16 = 40;
    const SLOW_STEP_CELLS: u16 = 2;

    /// Cell centres of one sine S curve centred on `row`, `step` cells apart in x.
    fn s_curve_cells(row: u16, step: u16) -> Vec<starboard::canvas::CellPos> {
        let count = S_CURVE_LENGTH.checked_div(step).unwrap_or(0);
        (0..=count)
            .map(|index| {
                let offset = index.saturating_mul(step);
                let progress = f32::from(offset) / f32::from(S_CURVE_LENGTH);
                let lift = (std::f32::consts::TAU * progress).sin();
                let y = S_CURVE_AMPLITUDE.mul_add(lift, f32::from(row));
                let x = S_CURVE_START.saturating_add(offset);
                let cell = Vec2::new(f32::from(x), y).round().as_uvec2();
                starboard::canvas::CellPos {
                    x: u16::try_from(cell.x).unwrap_or(0),
                    y: u16::try_from(cell.y).unwrap_or(0),
                }
            })
            .collect()
    }

    /// Paints one stroke through `cells` the way the drawer does: the same lookahead window and
    /// spline flattening, then each op is applied to the canvas.
    fn paint_spline(
        canvas: &mut starboard::canvas::Canvas,
        brush: starboard::input::OwnBrush,
        cells: &[starboard::canvas::CellPos],
        stroke_rank: u32,
        author: u16,
    ) {
        let action = starboard::canvas::DrawAction::Paint(brush.color);
        let mut latest = starboard::input::LatestPaintedCell::default();
        let mut points = cells
            .iter()
            .map(|cell| starboard::canvas::cell_centre(*cell));
        let mut lists = Vec::new();
        if let Some(first) = points.next() {
            starboard::input::start_stroke(&mut latest, first, action, stroke_rank);
            lists.push(vec![first]);
        }
        for point in points {
            lists.extend(starboard::input::advance_stroke(&mut latest, point));
        }
        lists.extend(starboard::input::finish_stroke(&mut latest));
        for points in lists {
            let op = starboard::input::draw_op(brush, action, points, stroke_rank);
            canvas.apply(
                OpMeta {
                    lamport: stroke_rank,
                    author,
                },
                &op,
            );
        }
    }

    /// The case brush at the edge close-up size: big enough that the swept edge is readable.
    fn big_case_brush(world: &World) -> starboard::input::OwnBrush {
        starboard::input::OwnBrush {
            size_index: EDGE_CLOSE_UP_SIZE_INDEX,
            ..brush_of(world.resource::<CaseBrush>().0)
        }
    }

    /// One S curve with a big brush, sampled every 40 cells: the far-apart points a fast
    /// movement produces.
    fn paint_fast_broad_stroke(world: &mut World) {
        let brush = big_case_brush(world);
        let cells = s_curve_cells(450, FAST_STEP_CELLS);
        let mut canvas = world.resource_mut::<starboard::canvas::Canvas>();
        paint_spline(&mut canvas, brush, &cells, 1201, 1);
    }

    /// The same S curve twice: sampled every 2 cells on top, every 40 cells below.
    fn paint_slow_and_fast(world: &mut World) {
        let brush = big_case_brush(world);
        let slow = s_curve_cells(250, SLOW_STEP_CELLS);
        let fast = s_curve_cells(650, FAST_STEP_CELLS);
        let mut canvas = world.resource_mut::<starboard::canvas::Canvas>();
        paint_spline(&mut canvas, brush, &slow, 1301, 1);
        paint_spline(&mut canvas, brush, &fast, 1302, 2);
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
                lele_bevy_preview::scene::State {
                    label: String::from("crossing strokes"),
                    apply: paint_crossing_strokes,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("edge close-up"),
                    apply: paint_edge_close_up,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("fast broad stroke"),
                    apply: paint_fast_broad_stroke,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("slow vs fast"),
                    apply: paint_slow_and_fast,
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

    const FIRST_SCRIPTED_CELL: starboard::canvas::CellPos =
        starboard::canvas::CellPos { x: 200, y: 300 };

    fn scripted_ops() -> Vec<starboard::canvas::DrawOp> {
        let brush = starboard::input::OwnBrush::default();
        (0_u16..6)
            .map(|index| {
                let x = 200_u16.saturating_add(index.saturating_mul(60));
                let y = 300_u16.saturating_add(index.saturating_mul(30));
                starboard::input::draw_op(
                    brush,
                    starboard::canvas::DrawAction::Paint(brush.color),
                    vec![
                        starboard::canvas::cell_centre(starboard::canvas::CellPos { x, y }),
                        starboard::canvas::cell_centre(starboard::canvas::CellPos {
                            x: x.saturating_add(80),
                            y: y.saturating_add(40),
                        }),
                    ],
                    u32::from(index).saturating_add(1),
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
                .write_message(starboard::input::PendingLocalDraw(op.clone()));
            apps[0].update();
        }

        let started = std::time::Instant::now();
        loop {
            for app in &mut apps {
                app.update();
            }
            let hashes: Vec<u64> = apps.iter().map(canvas_hash).collect();
            if hashes.first() == hashes.last() {
                for app in &apps {
                    let cell = starboard::canvas::read_cell(
                        app.world().resource::<starboard::canvas::Canvas>(),
                        FIRST_SCRIPTED_CELL,
                    );
                    assert!(
                        cell.top.color.is_some(),
                        "converged on an empty canvas: {hashes:?}"
                    );
                }
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
                    .write_message(starboard::input::PendingLocalDraw(op.clone()));
                app.update();
            }
        }

        let started = std::time::Instant::now();
        loop {
            for app in &mut apps {
                app.update();
            }
            let hashes: Vec<u64> = apps.iter().map(canvas_hash).collect();
            if hashes.first() == hashes.last() {
                for app in &apps {
                    let cell = starboard::canvas::read_cell(
                        app.world().resource::<starboard::canvas::Canvas>(),
                        FIRST_SCRIPTED_CELL,
                    );
                    assert!(
                        cell.top.color.is_some(),
                        "converged on an empty canvas: {hashes:?}"
                    );
                }
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
                .write_message(starboard::input::PendingLocalDraw(op.clone()));
            apps[1].update();
            apps[0]
                .world_mut()
                .write_message(starboard::input::PendingLocalDraw(op.clone()));
            apps[0].update();
        }

        let started = std::time::Instant::now();
        loop {
            for app in &mut apps {
                app.update();
            }
            let hashes: Vec<u64> = apps.iter().map(canvas_hash).collect();
            if hashes.first() == hashes.last() {
                for app in &apps {
                    let cell = starboard::canvas::read_cell(
                        app.world().resource::<starboard::canvas::Canvas>(),
                        FIRST_SCRIPTED_CELL,
                    );
                    assert!(
                        cell.top.color.is_some(),
                        "converged on an empty canvas: {hashes:?}"
                    );
                }
                return;
            }
            assert!(
                started.elapsed() <= CONVERGE_BUDGET,
                "canvases did not converge: {hashes:?}"
            );
            tokio::time::sleep(test_harness::SETTLE).await;
        }
    }

    fn spawn_window(app: &mut App) {
        app.world_mut().spawn((
            PrimaryWindow,
            Window {
                resolution: (1024, 1024).into(),
                ..default()
            },
        ));
    }

    fn cell_point(cell: starboard::canvas::CellPos) -> Vec2 {
        Vec2::new(f32::from(cell.x), f32::from(cell.y))
    }

    fn window_size(world: &mut World) -> Vec2 {
        world
            .query_filtered::<&Window, With<PrimaryWindow>>()
            .single(world)
            .map_or(Vec2::splat(1024.0), Window::size)
    }

    fn set_cursor(world: &mut World, board_point: Vec2) {
        let view = starboard::render::CanvasView::fit(window_size(world));
        let half_cell = Vec2::splat(0.5).mul_add(Vec2::splat(view.scale), view.offset);
        let screen = board_point.mul_add(Vec2::splat(view.scale), half_cell);
        let mut windows = world.query_filtered::<&mut Window, With<PrimaryWindow>>();
        if let Ok(mut window) = windows.single_mut(world) {
            window.set_cursor_position(Some(screen));
        }
    }

    fn start_held_stroke(app: &mut App, start: starboard::canvas::CellPos) {
        set_cursor(app.world_mut(), cell_point(start));
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        app.update();
    }

    fn drag_held(app: &mut App, cells: &[starboard::canvas::CellPos]) {
        for cell in cells {
            set_cursor(app.world_mut(), cell_point(*cell));
            app.update();
        }
    }

    fn release_left(app: &mut App) {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .reset_all();
    }

    fn held(app: &App) -> bool {
        app.world()
            .resource::<ButtonInput<MouseButton>>()
            .pressed(MouseButton::Left)
    }

    fn stroke_path() -> Vec<starboard::canvas::CellPos> {
        [
            (100_u16, 100_u16),
            (120, 104),
            (140, 110),
            (160, 118),
            (180, 124),
            (200, 130),
            (220, 140),
        ]
        .into_iter()
        .map(|(x, y)| starboard::canvas::CellPos { x, y })
        .collect()
    }

    fn cell_painted(
        app: &App,
        cell: starboard::canvas::CellPos,
        color: starboard::identity::PaletteColor,
    ) -> bool {
        starboard::canvas::read_cell(app.world().resource::<starboard::canvas::Canvas>(), cell)
            .top
            .color
            == Some(color)
    }

    fn board_pixel(app: &mut App, cell: starboard::canvas::CellPos) -> Option<[u8; 4]> {
        let world = app.world_mut();
        let handle = world
            .query_filtered::<&Sprite, With<starboard::render::BoardSprite>>()
            .single(world)
            .ok()?
            .image
            .clone();
        let image = world.resource::<Assets<Image>>().get(&handle)?;
        let data = image.data.as_ref()?;
        let row = usize::from(cell.y).saturating_mul(starboard::canvas::CANVAS_SIDE);
        let start = row.saturating_add(usize::from(cell.x)).saturating_mul(4);
        data.get(start..start.saturating_add(4))?.try_into().ok()
    }

    fn cell_shown(
        app: &mut App,
        cell: starboard::canvas::CellPos,
        color: starboard::identity::PaletteColor,
    ) -> bool {
        cell_painted(app, cell, color)
            && board_pixel(app, cell) == Some(starboard::render::palette_rgba(color))
    }

    fn pump_until_shown(
        apps: &mut [App],
        viewer: usize,
        cells: &[starboard::canvas::CellPos],
        color: starboard::identity::PaletteColor,
    ) {
        let started = std::time::Instant::now();
        loop {
            for app in apps.iter_mut() {
                app.update();
            }
            let Some(view) = apps.get_mut(viewer) else {
                panic!("no viewer app");
            };
            if cells.iter().all(|cell| cell_shown(view, *cell, color)) {
                return;
            }
            assert!(
                started.elapsed() <= CONVERGE_BUDGET,
                "viewer never showed the held stroke"
            );
            std::thread::sleep(test_harness::SETTLE);
        }
    }

    fn joined_pair(game: &identity::GameId, room: &identity::RoomName) -> Vec<App> {
        let mut apps = vec![
            test_harness::node_with(game, None),
            test_harness::node_with(game, None),
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
            test_harness::join(app, room, others);
        }
        apps
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_held_stroke_is_on_the_drawers_canvas_and_board_before_release() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-live");
        let mut apps = joined_pair(&game, &room);
        for _ in 0..50 {
            for app in &mut apps {
                app.update();
            }
            tokio::time::sleep(test_harness::SETTLE).await;
        }
        spawn_window(&mut apps[0]);

        let color = apps[0]
            .world()
            .resource::<starboard::input::OwnBrush>()
            .color;
        let path = stroke_path();
        let (&start, rest) = path.split_first().expect("a path");
        start_held_stroke(&mut apps[0], start);
        drag_held(&mut apps[0], rest);

        let (_, shown_while_held) = path.split_last().expect("a path");
        pump_until_shown(&mut apps, 0, shown_while_held, color);
        assert!(
            held(&apps[0]),
            "the stroke is still held when it is checked"
        );
        release_left(&mut apps[0]);
        pump_until_shown(&mut apps, 0, &path, color);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_held_stroke_shows_on_the_other_peer_before_release_and_its_extension_too() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-live");
        let mut apps = joined_pair(&game, &room);
        for _ in 0..50 {
            for app in &mut apps {
                app.update();
            }
            tokio::time::sleep(test_harness::SETTLE).await;
        }
        spawn_window(&mut apps[0]);

        let color = apps[0]
            .world()
            .resource::<starboard::input::OwnBrush>()
            .color;
        let path = stroke_path();
        let (&start, rest) = path.split_first().expect("a path");
        let (partial, extension) = rest.split_at(3);
        start_held_stroke(&mut apps[0], start);
        drag_held(&mut apps[0], partial);

        let (seen_so_far, _) = path.split_at(3);
        pump_until_shown(&mut apps, 1, seen_so_far, color);
        assert!(held(&apps[0]), "peer A still holds the button");

        drag_held(&mut apps[0], extension);
        let (_, shown_while_held) = path.split_last().expect("a path");
        pump_until_shown(&mut apps, 1, shown_while_held, color);
        assert!(held(&apps[0]), "peer A still holds the button");

        release_left(&mut apps[0]);
        let started = std::time::Instant::now();
        loop {
            for app in &mut apps {
                app.update();
            }
            let hashes: Vec<u64> = apps.iter().map(canvas_hash).collect();
            if hashes.first() == hashes.last() {
                return;
            }
            assert!(
                started.elapsed() <= CONVERGE_BUDGET,
                "canvases did not converge after release: {hashes:?}"
            );
            tokio::time::sleep(test_harness::SETTLE).await;
        }
    }

    const STROKE_SUBSTEPS: u16 = 6;
    const STROKE_SEGMENTS: u16 = 8;
    const STROKE_FRAMES: u32 = 48;
    const STROKE_FPS: u32 = 30;
    const STROKE_ORIGIN: starboard::canvas::CellPos = starboard::canvas::CellPos { x: 300, y: 520 };

    fn curve_cell(index: u16) -> starboard::canvas::CellPos {
        let (dx, dy) = CURVE.get(usize::from(index)).copied().unwrap_or((0, 0));
        starboard::canvas::CellPos {
            x: STROKE_ORIGIN.x.saturating_add(dx),
            y: STROKE_ORIGIN.y.saturating_add_signed(dy),
        }
    }

    fn curve_point(step: u16) -> Vec2 {
        let segment = step
            .checked_div(STROKE_SUBSTEPS)
            .unwrap_or(0)
            .min(STROKE_SEGMENTS.saturating_sub(1));
        let within = step.saturating_sub(segment.saturating_mul(STROKE_SUBSTEPS));
        let from = cell_point(curve_cell(segment));
        let to = cell_point(curve_cell(segment.saturating_add(1)));
        from.lerp(to, f32::from(within) / f32::from(STROKE_SUBSTEPS))
    }

    #[derive(Resource, Default)]
    struct StrokeClock {
        started_at: Option<std::time::Duration>,
    }

    fn start_stroke_clock(world: &mut World) {
        world.init_resource::<StrokeClock>();
    }

    fn drive_stroke(world: &mut World) {
        let elapsed = world.resource::<Time>().elapsed();
        let frame = {
            let mut clock = world.resource_mut::<StrokeClock>();
            let started_at = *clock.started_at.get_or_insert(elapsed);
            let since_ms = elapsed.saturating_sub(started_at).as_millis();
            let rounded = since_ms
                .saturating_mul(u128::from(STROKE_FPS))
                .saturating_add(500)
                / 1000;
            u32::try_from(rounded).unwrap_or(u32::MAX)
        };
        let step = u16::try_from(frame.min(STROKE_FRAMES)).unwrap_or(u16::MAX);
        set_cursor(world, curve_point(step));
        if frame == 0 {
            world
                .resource_mut::<ButtonInput<MouseButton>>()
                .press(MouseButton::Left);
        }
    }

    fn apply_pending_strokes(world: &mut World) {
        let ops = std::mem::take(
            &mut world
                .resource_mut::<PendingOps<starboard::canvas::Canvas>>()
                .ops,
        );
        let mut canvas = world.resource_mut::<starboard::canvas::Canvas>();
        for op in ops {
            canvas.apply(
                OpMeta {
                    lamport: op.stroke_rank,
                    author: 1,
                },
                &op,
            );
        }
    }

    fn build_star_stroke(app: &mut App) {
        preview_build(app, STAR_YELLOW);
        app.add_systems(
            Update,
            (
                drive_stroke
                    .run_if(resource_exists::<StrokeClock>)
                    .before(starboard::input::bevy_systems::track_cursor),
                apply_pending_strokes
                    .run_if(resource_exists::<StrokeClock>)
                    .after(starboard::input::bevy_systems::send_local_draws),
            ),
        );
    }

    fn star_stroke_scene(
        timeline: Option<lele_bevy_preview::scene::Timeline>,
    ) -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("starboard_star_stroke"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: build_star_stroke,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("empty board"),
                apply: preview_noop,
            }],
            timeline,
        }
    }

    #[test]
    fn star_stroke_drawing_mp4_preview() {
        let timeline = lele_bevy_preview::scene::Timeline {
            label: String::from("star_stroke_drawn"),
            frames: STROKE_FRAMES.saturating_add(1),
            fps: STROKE_FPS,
            apply: start_stroke_clock,
        };
        star_stroke_scene(Some(timeline))
            .run(&config())
            .expect("preview");
    }
}
