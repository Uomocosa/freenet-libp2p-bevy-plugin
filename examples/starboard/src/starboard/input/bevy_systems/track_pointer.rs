use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use freenet_libp2p_bevy_plugin::{room_sync, ui};

use crate::starboard;

pub fn track_pointer(
    buttons: Res<ButtonInput<MouseButton>>,
    menu: Option<Res<ui::discovery::EscMenuState>>,
    cursor: Res<starboard::input::CursorPoint>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    brush: Res<starboard::input::OwnBrush>,
    clock: Res<room_sync::LamportClock>,
    mut latest_painted: ResMut<starboard::input::LatestPaintedCell>,
    mut pending: ResMut<starboard::input::PendingPressState>,
    mut draws: MessageWriter<starboard::input::PendingLocalDraw>,
) {
    let buttons = buttons.into_inner();
    let clock = clock.into_inner();
    let cursor = cursor.into_inner();
    let time = time.into_inner();
    let brush = *brush.into_inner();
    let latest: &mut starboard::input::LatestPaintedCell = &mut latest_painted;
    let is_menu_open = menu.is_some_and(|menu| **menu);
    let held = [MouseButton::Right, MouseButton::Left]
        .into_iter()
        .find(|button| starboard::input::HeldButton(*button).is_held(buttons));
    let (Some(button), false) = (held, is_menu_open) else {
        latest.flush_stroke(brush, &mut draws);
        **pending = None;
        return;
    };
    let Some(current) = **cursor else {
        return;
    };
    let Some(screen) = windows.single().ok().and_then(Window::cursor_position) else {
        return;
    };
    let now = time.elapsed_secs();
    let action = if button == MouseButton::Right {
        starboard::canvas::DrawAction::Erase
    } else {
        starboard::canvas::DrawAction::Paint(brush.color)
    };
    if latest.cell.is_some() && latest.action != Some(action) {
        latest.flush_stroke(brush, &mut draws);
    }
    let mut lists = Vec::new();
    let press = (**pending).take();
    let had_press = press.is_some();
    match press.filter(|press| press.button == button) {
        Some(mut press) => {
            press.record(current);
            if press.has_become_stroke(screen, now) {
                for point in press.path.iter().skip(1) {
                    lists.extend(starboard::input::advance_stroke(latest, *point));
                }
            } else {
                **pending = Some(press);
            }
        }
        None if !had_press && latest.cell.is_some() => {
            lists.extend(starboard::input::advance_stroke(latest, current));
        }
        None if buttons.just_pressed(button) => {
            let stroke_rank = clock
                .saturating_add(1)
                .max(latest.stroke_rank.saturating_add(1));
            starboard::input::start_stroke(latest, current, action, stroke_rank);
            lists.push(vec![current]);
            **pending = Some(starboard::input::PendingPress::begin(
                button,
                current,
                screen,
                now,
                stroke_rank,
            ));
        }
        None => latest.cell = None,
    }
    starboard::input::PendingLocalDraw::write_lists(
        &mut draws,
        brush,
        action,
        latest.stroke_rank,
        lists,
    );
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use bevy::window::PrimaryWindow;
    use freenet_libp2p_bevy_plugin::room_sync;
    use freenet_libp2p_bevy_plugin::ui;

    use std::ops::{Mul, Sub};

    use super::track_pointer;
    use crate::starboard;
    use starboard::canvas::{DrawAction, DrawOp, SubCellPos};

    fn app_with_pointer() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        app.init_resource::<room_sync::PendingOps<starboard::canvas::Canvas>>();
        app.insert_resource(starboard::render::CanvasView::fit(Vec2::splat(1024.0)));
        app.insert_resource(starboard::input::OwnBrush::default());
        app.init_resource::<room_sync::LamportClock>();
        app.init_resource::<starboard::input::LatestPaintedCell>();
        app.init_resource::<starboard::input::PendingPressState>();
        app.init_resource::<starboard::input::CursorCell>();
        app.init_resource::<starboard::input::CursorPoint>();
        app.init_resource::<ButtonInput<MouseButton>>();
        app.add_message::<starboard::input::PendingLocalDraw>();
        app.world_mut().spawn((
            PrimaryWindow,
            Window {
                resolution: (1024, 1024).into(),
                ..default()
            },
        ));
        app.add_systems(
            Update,
            (
                starboard::input::bevy_systems::track_cursor,
                track_pointer,
                starboard::input::bevy_systems::send_local_draws,
            )
                .chain(),
        );
        app
    }

    fn point_at(app: &mut App, screen: Vec2) {
        let mut windows = app
            .world_mut()
            .query_filtered::<&mut Window, With<PrimaryWindow>>();
        let mut window = windows
            .single_mut(app.world_mut())
            .expect("one primary window");
        window.set_cursor_position(Some(screen));
    }

    fn set_frame_step(app: &mut App, millis: u64) {
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            millis,
        )));
    }

    fn press(app: &mut App, button: MouseButton) {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(button);
    }

    fn release_all(app: &mut App) {
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .reset_all();
    }

    fn sent(app: &App) -> Vec<DrawOp> {
        app.world()
            .resource::<room_sync::PendingOps<starboard::canvas::Canvas>>()
            .ops
            .clone()
    }

    /// Stands in for core's `write_ops`, which ticks the clock once per op it writes.
    fn tick_once_per_sent_op(app: &mut App) {
        let count = sent(app).len();
        let mut clock = app.world_mut().resource_mut::<room_sync::LamportClock>();
        for _ in 0..count {
            clock.tick();
        }
    }

    /// Every sent point in order. Ops share their boundary point, so repeats are dropped.
    fn sent_path(app: &App) -> Vec<SubCellPos> {
        let mut path: Vec<SubCellPos> = Vec::new();
        for point in sent(app).iter().flat_map(|op| op.points.iter().copied()) {
            if path.last() != Some(&point) {
                path.push(point);
            }
        }
        path
    }

    /// A whole screen pixel at scale 1 is a sub-cell point of `8 * pixel`, so the expectations
    /// below stay readable.
    const fn pixel(x: u32, y: u32) -> SubCellPos {
        SubCellPos {
            x: x.saturating_mul(8),
            y: y.saturating_mul(8),
        }
    }

    #[test]
    fn mouse_press_sends_one_stamp() {
        let mut app = app_with_pointer();
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();

        let ops = sent(&app);
        assert_eq!(ops.len(), 1);
        let stamp = pixel(100, 100);
        assert_eq!(ops.first().map(|op| op.points.clone()), Some(vec![stamp]));
        assert!(matches!(
            ops.first().map(|op| op.action),
            Some(DrawAction::Paint(_))
        ));
    }

    #[test]
    fn a_click_is_one_dab() {
        let mut app = app_with_pointer();
        set_frame_step(&mut app, 50);
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();
        release_all(&mut app);
        app.update();

        assert_eq!(sent_path(&app), vec![pixel(100, 100)]);
    }

    #[test]
    fn moving_past_the_slop_turns_the_press_into_a_stroke() {
        let mut app = app_with_pointer();
        set_frame_step(&mut app, 50);
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();
        point_at(&mut app, Vec2::new(110.0, 100.0));
        app.update();

        assert_eq!(
            sent_path(&app),
            vec![pixel(100, 100)],
            "the segment waits for a point after the move"
        );
        release_all(&mut app);
        app.update();
        assert_eq!(
            sent_path(&app).last(),
            Some(&pixel(110, 100)),
            "release flushes the segment"
        );
    }

    #[test]
    fn holding_past_the_time_limit_turns_the_press_into_a_stroke() {
        let mut app = app_with_pointer();
        set_frame_step(&mut app, 175);
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();
        app.update();
        assert_eq!(sent(&app).len(), 1, "still a dot at 0.175 s");
        app.update();
        assert_eq!(sent(&app).len(), 1, "a still stroke adds no segment");

        point_at(&mut app, Vec2::new(200.0, 100.0));
        app.update();
        assert_eq!(sent(&app).len(), 1, "the new point waits for its successor");
        release_all(&mut app);
        app.update();
        assert_eq!(sent_path(&app).last(), Some(&pixel(200, 100)));
    }

    #[test]
    fn a_small_move_within_the_time_limit_stays_a_dot() {
        let mut app = app_with_pointer();
        set_frame_step(&mut app, 145);
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();
        app.update();
        point_at(&mut app, Vec2::new(103.0, 100.0));
        app.update();

        assert_eq!(sent_path(&app), vec![pixel(100, 100)]);
    }

    #[test]
    fn the_path_before_the_threshold_is_painted_when_the_press_becomes_a_stroke() {
        let mut app = app_with_pointer();
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();
        for screen in [(103.0, 100.0), (103.0, 102.0), (100.0, 110.0)] {
            point_at(&mut app, Vec2::new(screen.0, screen.1));
            app.update();
        }

        assert_eq!(
            sent_path(&app).last(),
            Some(&pixel(103, 102)),
            "the newest recorded segment is still waiting for its successor"
        );
        release_all(&mut app);
        app.update();
        assert_eq!(
            sent_path(&app).last(),
            Some(&pixel(100, 110)),
            "release sends the tail"
        );
        assert_eq!(sent_path(&app).first(), Some(&pixel(100, 100)));
    }

    #[test]
    fn a_sub_pixel_move_still_extends_the_stroke() {
        let mut app = app_with_pointer();
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();
        point_at(&mut app, Vec2::new(110.0, 100.0));
        app.update();
        point_at(&mut app, Vec2::new(110.4, 100.0));
        app.update();
        release_all(&mut app);
        app.update();

        assert_eq!(
            sent_path(&app).last(),
            Some(&SubCellPos { x: 883, y: 800 }),
            "a move of a fraction of a cell is sent at sub-cell precision"
        );
    }

    #[test]
    fn the_right_button_erases_with_the_same_dot_and_stroke_rules() {
        let mut app = app_with_pointer();
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();
        release_all(&mut app);
        press(&mut app, MouseButton::Right);
        app.update();
        point_at(&mut app, Vec2::new(110.0, 100.0));
        app.update();
        release_all(&mut app);
        app.update();

        let ops = sent(&app);
        assert_eq!(ops.len(), 3);
        assert_eq!(ops.get(1).map(|op| op.action), Some(DrawAction::Erase));
        assert_eq!(
            ops.get(2).map(|op| op.action),
            Some(DrawAction::Erase),
            "the flushed tail keeps the erase action"
        );
        assert_eq!(
            ops.get(2).and_then(|op| op.points.last().copied()),
            Some(pixel(110, 100))
        );
    }

    #[test]
    fn dragging_sends_the_segment_and_release_clears_the_tail() {
        let mut app = app_with_pointer();
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();
        point_at(&mut app, Vec2::new(200.0, 100.0));
        app.update();

        assert_eq!(sent_path(&app), vec![pixel(100, 100)]);

        release_all(&mut app);
        app.update();
        assert_eq!(sent_path(&app).first(), Some(&pixel(100, 100)));
        assert_eq!(sent_path(&app).last(), Some(&pixel(200, 100)));
        let latest = app
            .world()
            .resource::<starboard::input::LatestPaintedCell>();
        assert!(latest.cell.is_none());
    }

    #[test]
    fn an_open_esc_menu_stops_the_painting() {
        let mut app = app_with_pointer();
        app.insert_resource(ui::discovery::EscMenuState(true));
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();
        assert_eq!(sent(&app).len(), 0, "the menu blocks the painting");
        **app
            .world_mut()
            .resource_mut::<ui::discovery::EscMenuState>() = false;
        release_all(&mut app);
        press(&mut app, MouseButton::Left);
        app.update();
        point_at(&mut app, Vec2::new(200.0, 100.0));
        app.update();
        release_all(&mut app);
        app.update();
        assert_eq!(
            sent(&app).first().map(|op| op.points.clone()),
            Some(vec![pixel(100, 100)]),
            "the first op after the menu is a fresh dab"
        );
        assert_eq!(
            sent_path(&app).last(),
            Some(&pixel(200, 100)),
            "painting resumes from the fresh cell, not from the pre-menu tail"
        );
    }

    #[test]
    fn every_op_of_one_press_carries_the_same_stroke_rank() {
        let mut app = app_with_pointer();
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();
        for screen in [
            (103.0, 100.0),
            (103.0, 102.0),
            (100.0, 110.0),
            (160.0, 110.0),
        ] {
            point_at(&mut app, Vec2::new(screen.0, screen.1));
            app.update();
        }

        let ops = sent(&app);
        assert!(ops.len() >= 4, "dab, catch-up and live segments");
        let rank = ops.first().map(|op| op.stroke_rank);
        assert!(rank.is_some_and(|rank| rank > 0));
        assert!(ops.iter().all(|op| Some(op.stroke_rank) == rank));
    }

    #[test]
    fn a_later_press_ranks_above_an_earlier_one() {
        let mut app = app_with_pointer();
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();
        release_all(&mut app);
        app.update();
        tick_once_per_sent_op(&mut app);

        point_at(&mut app, Vec2::new(300.0, 300.0));
        press(&mut app, MouseButton::Left);
        app.update();

        let ranks: Vec<u32> = sent(&app).iter().map(|op| op.stroke_rank).collect();
        assert_eq!(ranks.len(), 2);
        assert!(
            ranks
                .get(1)
                .is_some_and(|later| ranks.first().is_some_and(|first| later > first))
        );
    }

    #[test]
    fn presses_with_no_clock_tick_between_them_rank_strictly_increasing() {
        let mut app = app_with_pointer();
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();
        release_all(&mut app);
        app.update();
        point_at(&mut app, Vec2::new(300.0, 300.0));
        press(&mut app, MouseButton::Left);
        app.update();

        let ranks: Vec<u32> = sent(&app).iter().map(|op| op.stroke_rank).collect();
        assert_eq!(ranks.len(), 2);
        assert!(
            ranks
                .get(1)
                .is_some_and(|later| ranks.first().is_some_and(|first| later > first))
        );
    }

    #[test]
    fn a_press_after_observing_a_remote_op_ranks_above_it() {
        let mut app = app_with_pointer();
        app.world_mut()
            .resource_mut::<room_sync::LamportClock>()
            .observe(50);
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();

        let rank = sent(&app).first().map(|op| op.stroke_rank);
        assert!(rank.is_some_and(|rank| rank > 50));
    }

    const BIG_BRUSH_SIZE_INDEX: u8 = 7;
    const ARC_CENTRE: f32 = 400.0;
    const ARC_RADIUS_PX: f32 = 100.0;

    fn arc_point(angle: f32) -> Vec2 {
        Vec2::new(
            ARC_RADIUS_PX.mul_add(angle.cos(), ARC_CENTRE),
            ARC_RADIUS_PX.mul_add(angle.sin(), ARC_CENTRE),
        )
    }

    /// Presses at the first point and drags through the rest, one update per point, then
    /// releases and runs one more update so the tail is flushed.
    fn paint_through(app: &mut App, screens: &[Vec2]) {
        for (index, screen) in screens.iter().enumerate() {
            point_at(app, *screen);
            if index == 0 {
                press(app, MouseButton::Left);
            }
            app.update();
        }
        release_all(app);
        app.update();
    }

    fn painted_cells(ops: &[DrawOp]) -> std::collections::BTreeSet<starboard::canvas::CellPos> {
        ops.iter().flat_map(starboard::canvas::rasterize).collect()
    }

    /// The sub-cell point under a screen pixel at scale 1, rounded to the nearest sub-cell.
    fn sub_cell_of(screen: Vec2) -> SubCellPos {
        let scaled = Vec2::mul(screen, 8.0).round().as_uvec2();
        SubCellPos {
            x: scaled.x,
            y: scaled.y,
        }
    }

    fn as_vec2(point: SubCellPos) -> Vec2 {
        bevy::math::UVec2::new(point.x, point.y).as_vec2()
    }

    /// The largest turn, in degrees, between two consecutive chords of a polyline.
    fn max_turn_degrees(points: &[Vec2]) -> f32 {
        points
            .iter()
            .zip(points.iter().skip(1))
            .zip(points.iter().skip(2))
            .map(|((a, b), c)| b.sub(*a).angle_to(c.sub(*b)).abs().to_degrees())
            .fold(0.0, f32::max)
    }

    /// The sent centreline, sampled no closer than two cells, so rounding to sub-cells does not
    /// read as a turn.
    fn two_cell_samples(path: &[SubCellPos]) -> Vec<Vec2> {
        let mut samples: Vec<SubCellPos> = Vec::new();
        for point in path.iter().copied() {
            let far_enough = samples
                .last()
                .is_none_or(|last| as_vec2(*last).distance(as_vec2(point)) >= 16.0);
            if far_enough {
                samples.push(point);
            }
        }
        samples.into_iter().map(as_vec2).collect()
    }

    fn neighbours_of(
        cell: starboard::canvas::CellPos,
        steps: &[(i32, i32)],
    ) -> impl Iterator<Item = starboard::canvas::CellPos> + '_ {
        steps.iter().filter_map(move |(dx, dy)| {
            let x = i32::from(cell.x).checked_add(*dx)?;
            let y = i32::from(cell.y).checked_add(*dy)?;
            Some(starboard::canvas::CellPos {
                x: u16::try_from(x).ok()?,
                y: u16::try_from(y).ok()?,
            })
        })
    }

    fn reached(
        start: starboard::canvas::CellPos,
        steps: &[(i32, i32)],
        is_open: impl Fn(starboard::canvas::CellPos) -> bool,
    ) -> std::collections::BTreeSet<starboard::canvas::CellPos> {
        let mut seen = std::collections::BTreeSet::from([start]);
        let mut queue = std::collections::VecDeque::from([start]);
        while let Some(cell) = queue.pop_front() {
            for next in neighbours_of(cell, steps) {
                if is_open(next) && seen.insert(next) {
                    queue.push_back(next);
                }
            }
        }
        seen
    }

    const FOUR_STEPS: [(i32, i32); 4] = [(1, 0), (-1, 0), (0, 1), (0, -1)];
    const EIGHT_STEPS: [(i32, i32); 8] = [
        (1, 0),
        (-1, 0),
        (0, 1),
        (0, -1),
        (1, 1),
        (1, -1),
        (-1, 1),
        (-1, -1),
    ];

    fn big_brush(
        shape: starboard::identity::Shape,
        color: starboard::identity::PaletteColor,
    ) -> starboard::input::OwnBrush {
        starboard::input::OwnBrush {
            shape,
            color,
            size_index: BIG_BRUSH_SIZE_INDEX,
        }
    }

    #[test]
    fn the_newest_segment_waits_for_its_successor_and_release_flushes_it() {
        let mut app = app_with_pointer();
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();
        point_at(&mut app, Vec2::new(140.0, 100.0));
        app.update();
        assert_eq!(sent_path(&app), vec![pixel(100, 100)]);

        point_at(&mut app, Vec2::new(180.0, 100.0));
        app.update();
        assert_eq!(sent_path(&app).last(), Some(&pixel(140, 100)));

        release_all(&mut app);
        app.update();
        assert_eq!(sent_path(&app).last(), Some(&pixel(180, 100)));
    }

    #[test]
    fn the_spline_passes_through_every_recorded_point() {
        let mut app = app_with_pointer();
        let screens = [
            Vec2::new(100.0, 100.0),
            Vec2::new(140.0, 130.0),
            Vec2::new(180.0, 100.0),
            Vec2::new(220.0, 130.0),
            Vec2::new(260.0, 100.0),
            Vec2::new(300.0, 130.0),
        ];
        paint_through(&mut app, &screens);
        let path = sent_path(&app);
        for screen in &screens {
            let expected = sub_cell_of(*screen);
            assert!(
                path.contains(&expected),
                "missing recorded point {expected:?}"
            );
        }
    }

    #[rstest::rstest]
    #[case::circle_red(starboard::identity::Shape::Circle, starboard::identity::PaletteColor(0), &FOUR_STEPS)]
    #[case::square_blue(starboard::identity::Shape::Square, starboard::identity::PaletteColor(6), &FOUR_STEPS)]
    #[case::star_yellow(starboard::identity::Shape::Star, starboard::identity::PaletteColor(11), &EIGHT_STEPS)]
    fn fast_broad_stroke_has_no_corners(
        #[case] shape: starboard::identity::Shape,
        #[case] color: starboard::identity::PaletteColor,
        #[case] joins: &[(i32, i32)],
    ) {
        let mut app = app_with_pointer();
        app.insert_resource(big_brush(shape, color));
        let angles: Vec<f32> = (0..12_u16).map(|index| 0.4 * f32::from(index)).collect();
        let screens: Vec<Vec2> = angles.iter().map(|angle| arc_point(*angle)).collect();

        let cursor_polygon = max_turn_degrees(&screens);
        assert!(
            cursor_polygon > 15.0,
            "the straight polygon through the cursor points turns {cursor_polygon} degrees: the control"
        );

        paint_through(&mut app, &screens);
        let path = sent_path(&app);
        let turn = max_turn_degrees(&two_cell_samples(&path));
        assert!(
            turn <= 15.0,
            "the centreline turns {turn} degrees between samples"
        );

        let cells = painted_cells(&sent(&app));
        let first = cells.first().copied().unwrap_or_default();
        let reached_cells = reached(first, joins, |cell| cells.contains(&cell));
        assert_eq!(reached_cells.len(), cells.len(), "the ribbon is one piece");
    }

    #[rstest::rstest]
    #[case::circle_red(
        starboard::identity::Shape::Circle,
        starboard::identity::PaletteColor(0)
    )]
    #[case::square_blue(
        starboard::identity::Shape::Square,
        starboard::identity::PaletteColor(6)
    )]
    #[case::star_yellow(
        starboard::identity::Shape::Star,
        starboard::identity::PaletteColor(11)
    )]
    fn slow_and_fast_strokes_paint_nearly_the_same_cells(
        #[case] shape: starboard::identity::Shape,
        #[case] color: starboard::identity::PaletteColor,
    ) {
        let total = 1.6_f32;
        let fast_angles: Vec<f32> = (0..=4_u16)
            .map(|index| total * f32::from(index) / 4.0)
            .collect();
        let slow_angles: Vec<f32> = (0..=80_u16)
            .map(|index| total * f32::from(index) / 80.0)
            .collect();

        let mut fast_app = app_with_pointer();
        fast_app.insert_resource(big_brush(shape, color));
        let fast_screens: Vec<Vec2> = fast_angles.iter().map(|a| arc_point(*a)).collect();
        paint_through(&mut fast_app, &fast_screens);
        let fast = painted_cells(&sent(&fast_app));

        let mut slow_app = app_with_pointer();
        slow_app.insert_resource(big_brush(shape, color));
        let slow_screens: Vec<Vec2> = slow_angles.iter().map(|a| arc_point(*a)).collect();
        paint_through(&mut slow_app, &slow_screens);
        let slow = painted_cells(&sent(&slow_app));

        let differing = fast.symmetric_difference(&slow).count();
        let painted = fast.union(&slow).count();
        assert!(
            differing.saturating_mul(100) < painted.saturating_mul(5),
            "{differing} of {painted} cells differ between the slow and the fast stroke"
        );
    }
}
