use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use freenet_libp2p_bevy_plugin::ui;

use crate::starboard;

pub fn track_pointer(
    buttons: Res<ButtonInput<MouseButton>>,
    menu: Option<Res<ui::discovery::EscMenuState>>,
    cursor: Res<starboard::input::CursorCell>,
    windows: Query<&Window, With<PrimaryWindow>>,
    time: Res<Time>,
    brush: Res<starboard::input::OwnBrush>,
    mut latest_painted: ResMut<starboard::input::LatestPaintedCell>,
    mut pending: ResMut<starboard::input::PendingPressState>,
    mut draws: MessageWriter<starboard::input::PendingLocalDraw>,
) {
    let buttons = buttons.into_inner();
    let cursor = cursor.into_inner();
    let time = time.into_inner();
    let brush = *brush.into_inner();
    let is_menu_open = menu.is_some_and(|menu| **menu);
    let held = [MouseButton::Right, MouseButton::Left]
        .into_iter()
        .find(|button| is_held(buttons, *button));
    let (Some(button), false) = (held, is_menu_open) else {
        **latest_painted = None;
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
    let current_pending = (**pending).take();
    let (strokes, next_pending, next_latest) = match (current_pending, **latest_painted) {
        (Some(mut press), _) if press.button == button => {
            press.record(current);
            if press.has_become_stroke(screen, now) {
                (press.segments(), None, Some(current))
            } else {
                (Vec::new(), Some(press), **latest_painted)
            }
        }
        (None, Some(last)) => (
            vec![starboard::canvas::Stroke {
                from: last,
                to: current,
            }],
            None,
            Some(current),
        ),
        _ if buttons.just_pressed(button) => (
            vec![starboard::canvas::Stroke {
                from: current,
                to: current,
            }],
            Some(starboard::input::PendingPress::begin(
                button, current, screen, now,
            )),
            Some(current),
        ),
        _ => (Vec::new(), None, None),
    };
    for stroke in strokes {
        draws.write(starboard::input::PendingLocalDraw(
            starboard::input::draw_op(brush, action, stroke),
        ));
    }
    **pending = next_pending;
    **latest_painted = next_latest;
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
    use starboard::canvas::{CellPos, DrawAction, DrawOp};

    fn app_with_pointer() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(Duration::ZERO));
        app.init_resource::<room_sync::PendingOps<starboard::canvas::Canvas>>();
        app.insert_resource(starboard::render::CanvasView::fit(Vec2::splat(1024.0)));
        app.insert_resource(starboard::input::OwnBrush::default());
        app.init_resource::<starboard::input::LatestPaintedCell>();
        app.init_resource::<starboard::input::PendingPressState>();
        app.init_resource::<starboard::input::CursorCell>();
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

    fn sent_segments(app: &App) -> Vec<(CellPos, CellPos)> {
        sent(app).iter().map(|op| (op.from, op.to)).collect()
    }

    const fn cell(x: u16, y: u16) -> CellPos {
        CellPos { x, y }
    }

    #[test]
    fn mouse_press_sends_one_stamp() {
        let mut app = app_with_pointer();
        point_at(&mut app, Vec2::new(100.0, 100.0));
        press(&mut app, MouseButton::Left);
        app.update();

        let ops = sent(&app);
        assert_eq!(ops.len(), 1);
        let stamp = cell(100, 100);
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

        assert_eq!(sent_segments(&app), vec![(cell(100, 100), cell(100, 100))]);
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
                (cell(100, 100), cell(100, 100)),
                (cell(100, 100), cell(110, 100))
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
            Some(&(cell(100, 100), cell(200, 100)))
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

        assert_eq!(sent_segments(&app), vec![(cell(100, 100), cell(100, 100))]);
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
                (cell(100, 100), cell(100, 100)),
                (cell(100, 100), cell(103, 100)),
                (cell(103, 100), cell(103, 102)),
                (cell(103, 102), cell(100, 110)),
            ]
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
            Some((DrawAction::Erase, cell(100, 100), cell(110, 100)))
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
                (cell(100, 100), cell(100, 100)),
                (cell(100, 100), cell(200, 100))
            ]
        );

        release_all(&mut app);
        app.update();
        let latest = app
            .world()
            .resource::<starboard::input::LatestPaintedCell>();
        assert!((**latest).is_none());
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
                (cell(100, 100), cell(100, 100)),
                (cell(100, 100), cell(200, 100))
            ],
            "painting resumes from the fresh cell, not from the pre-menu tail"
        );
    }
}
