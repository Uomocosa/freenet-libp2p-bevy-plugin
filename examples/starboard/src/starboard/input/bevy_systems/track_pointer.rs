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
    let is_menu_open = menu.is_some_and(|menu| **menu);
    let held = [MouseButton::Right, MouseButton::Left]
        .into_iter()
        .find(|button| is_held(buttons, *button));
    let (Some(button), false) = (held, is_menu_open) else {
        latest_painted.cell = None;
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
    let latest = *latest_painted;
    let current_pending = (**pending).take();
    let (strokes, next_pending, next_latest) = match (current_pending, latest.cell) {
        (Some(mut press), _) if press.button == button => {
            press.record(current);
            if press.has_become_stroke(screen, now) {
                let latched = starboard::input::LatestPaintedCell {
                    cell: Some(current),
                    stroke_rank: press.stroke_rank,
                };
                (press.segments(), None, latched)
            } else {
                (Vec::new(), Some(press), latest)
            }
        }
        (None, Some(last)) => (
            vec![starboard::canvas::Stroke {
                from: last,
                to: current,
            }],
            None,
            starboard::input::LatestPaintedCell {
                cell: Some(current),
                stroke_rank: latest.stroke_rank,
            },
        ),
        _ if buttons.just_pressed(button) => {
            let stroke_rank = clock
                .saturating_add(1)
                .max(latest.stroke_rank.saturating_add(1));
            (
                vec![starboard::canvas::Stroke {
                    from: current,
                    to: current,
                }],
                Some(starboard::input::PendingPress::begin(
                    button,
                    current,
                    screen,
                    now,
                    stroke_rank,
                )),
                starboard::input::LatestPaintedCell {
                    cell: Some(current),
                    stroke_rank,
                },
            )
        }
        _ => (
            Vec::new(),
            None,
            starboard::input::LatestPaintedCell {
                cell: None,
                stroke_rank: latest.stroke_rank,
            },
        ),
    };
    let stroke_rank = next_latest.stroke_rank;
    for stroke in strokes {
        draws.write(starboard::input::PendingLocalDraw(
            starboard::input::draw_op(brush, action, stroke, stroke_rank),
        ));
    }
    **pending = next_pending;
    *latest_painted = next_latest;
}

// needed helper: a click shorter than one frame still paints, so just_pressed counts as held
fn is_held(buttons: &ButtonInput<MouseButton>, button: MouseButton) -> bool {
    buttons.pressed(button) || buttons.just_pressed(button)
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use bevy::window::PrimaryWindow;
    use freenet_libp2p_bevy_plugin::room_sync;
    use freenet_libp2p_bevy_plugin::ui;

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

    fn sent_segments(app: &App) -> Vec<(SubCellPos, SubCellPos)> {
        sent(app).iter().map(|op| (op.from, op.to)).collect()
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
        assert_eq!(ops.first().map(|op| (op.from, op.to)), Some((stamp, stamp)));
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

        assert_eq!(
            sent_segments(&app),
            vec![(pixel(100, 100), pixel(100, 100))]
        );
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
            sent_segments(&app),
            vec![
                (pixel(100, 100), pixel(100, 100)),
                (pixel(100, 100), pixel(110, 100))
            ]
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
        assert_eq!(
            sent_segments(&app).last(),
            Some(&(pixel(100, 100), pixel(200, 100)))
        );
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

        assert_eq!(
            sent_segments(&app),
            vec![(pixel(100, 100), pixel(100, 100))]
        );
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
            sent_segments(&app),
            vec![
                (pixel(100, 100), pixel(100, 100)),
                (pixel(100, 100), pixel(103, 100)),
                (pixel(103, 100), pixel(103, 102)),
                (pixel(103, 102), pixel(100, 110)),
            ]
        );
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

        assert_eq!(
            sent_segments(&app).last(),
            Some(&(pixel(110, 100), SubCellPos { x: 883, y: 800 })),
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

        let ops = sent(&app);
        assert_eq!(ops.len(), 3);
        assert_eq!(ops.get(1).map(|op| op.action), Some(DrawAction::Erase));
        assert_eq!(
            ops.get(2).map(|op| (op.action, op.from, op.to)),
            Some((DrawAction::Erase, pixel(100, 100), pixel(110, 100)))
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

        assert_eq!(
            sent_segments(&app),
            vec![
                (pixel(100, 100), pixel(100, 100)),
                (pixel(100, 100), pixel(200, 100))
            ]
        );

        release_all(&mut app);
        app.update();
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
        assert_eq!(
            sent_segments(&app),
            vec![
                (pixel(100, 100), pixel(100, 100)),
                (pixel(100, 100), pixel(200, 100))
            ],
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
        assert!(ops.len() >= 5, "dab, catch-up and live segments");
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
}
