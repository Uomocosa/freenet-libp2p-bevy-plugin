use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::starboard;

pub fn track_pointer(
    buttons: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    view: Res<starboard::render::CanvasView>,
    brush: Res<starboard::input::OwnBrush>,
    mut latest_painted: ResMut<starboard::input::LatestPaintedCell>,
    mut draws: MessageWriter<starboard::input::PendingLocalDraw>,
) {
    let buttons = buttons.into_inner();
    let view = view.into_inner();
    let brush = brush.into_inner();
    let current = windows
        .single()
        .ok()
        .and_then(Window::cursor_position)
        .and_then(|cursor| starboard::input::cell_under_cursor(cursor, view));
    let is_erasing = is_held(buttons, MouseButton::Right);
    let is_painting = is_held(buttons, MouseButton::Left);
    if !is_erasing && !is_painting {
        **latest_painted = None;
        return;
    }
    let Some(current) = current else {
        return;
    };
    let from = (**latest_painted).unwrap_or(current);
    let action = if is_erasing {
        starboard::canvas::DrawAction::Erase
    } else {
        starboard::canvas::DrawAction::Paint(brush.color)
    };
    draws.write(starboard::input::PendingLocalDraw(
        starboard::input::draw_op(*brush, action, from, current),
    ));
    **latest_painted = Some(current);
}

// needed helper: a click shorter than one frame still paints, so just_pressed counts as held
fn is_held(buttons: &ButtonInput<MouseButton>, button: MouseButton) -> bool {
    buttons.pressed(button) || buttons.just_pressed(button)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;
    use freenet_libp2p_bevy_plugin::room_sync;

    use super::track_pointer;
    use crate::starboard;

    fn app_with_pointer() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<starboard::canvas::Canvas>();
        app.insert_resource(starboard::render::CanvasView::fit(Vec2::splat(1024.0)));
        app.insert_resource(starboard::input::OwnBrush::default());
        app.init_resource::<starboard::input::LatestPaintedCell>();
        app.init_resource::<room_sync::LamportClock>();
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
                track_pointer,
                starboard::input::bevy_systems::apply_local_draws,
            )
                .chain(),
        );
        app
    }

    fn set_cursor(app: &mut App, position: Vec2) {
        let mut windows = app
            .world_mut()
            .query_filtered::<&mut Window, With<PrimaryWindow>>();
        let mut window = windows
            .single_mut(app.world_mut())
            .expect("one primary window");
        window.set_cursor_position(Some(position));
    }

    fn is_painted(app: &App, x: u16, y: u16) -> bool {
        let canvas = app.world().resource::<starboard::canvas::Canvas>();
        starboard::canvas::read_cell(canvas, starboard::canvas::CellPos { x, y })
            .color
            .is_some()
    }

    #[test]
    fn mouse_press_paints_cells() {
        let mut app = app_with_pointer();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        set_cursor(&mut app, Vec2::new(100.0, 100.0));
        app.update();
        assert!(is_painted(&app, 100, 100));
        assert!(is_painted(&app, 90, 100));
    }

    #[test]
    fn dragging_paints_the_segment_and_release_clears_the_tail() {
        let mut app = app_with_pointer();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        set_cursor(&mut app, Vec2::new(100.0, 100.0));
        app.update();
        set_cursor(&mut app, Vec2::new(200.0, 100.0));
        app.update();
        assert!(is_painted(&app, 150, 100));

        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .reset_all();
        app.update();
        let latest = app
            .world()
            .resource::<starboard::input::LatestPaintedCell>();
        assert!((**latest).is_none());
    }

    #[test]
    fn right_button_erases() {
        let mut app = app_with_pointer();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        set_cursor(&mut app, Vec2::new(100.0, 100.0));
        app.update();
        assert!(is_painted(&app, 100, 100));

        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        app.update();
        assert!(!is_painted(&app, 100, 100));
    }

    #[test]
    fn a_still_cursor_records_one_stamp_per_frame() {
        let mut app = app_with_pointer();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        set_cursor(&mut app, Vec2::new(300.0, 300.0));
        app.update();
        app.update();
        let canvas = app.world().resource::<starboard::canvas::Canvas>();
        let cell =
            starboard::canvas::read_cell(canvas, starboard::canvas::CellPos { x: 300, y: 300 });
        assert!(cell.color.is_some());
    }
}
