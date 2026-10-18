use bevy::prelude::*;

use crate::starboard;

pub fn track_pointer(
    buttons: Res<ButtonInput<MouseButton>>,
    cursor: Res<starboard::input::CursorCell>,
    brush: Res<starboard::input::OwnBrush>,
    mut latest_painted: ResMut<starboard::input::LatestPaintedCell>,
    mut draws: MessageWriter<starboard::input::PendingLocalDraw>,
) {
    let buttons = buttons.into_inner();
    let cursor = cursor.into_inner();
    let brush = brush.into_inner();
    let current = **cursor;
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
    use freenet_libp2p_bevy_plugin::room_sync;

    use super::track_pointer;
    use crate::starboard;

    fn app_with_pointer() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.init_resource::<room_sync::PendingOps<starboard::canvas::Canvas>>();
        app.insert_resource(starboard::render::CanvasView::fit(Vec2::splat(1024.0)));
        app.insert_resource(starboard::input::OwnBrush::default());
        app.init_resource::<starboard::input::LatestPaintedCell>();
        app.init_resource::<starboard::input::CursorCell>();
        app.init_resource::<ButtonInput<MouseButton>>();
        app.add_message::<starboard::input::PendingLocalDraw>();
        app.add_systems(
            Update,
            (
                track_pointer,
                starboard::input::bevy_systems::send_local_draws,
            )
                .chain(),
        );
        app
    }

    fn set_cursor(app: &mut App, cell: Option<starboard::canvas::CellPos>) {
        **app
            .world_mut()
            .resource_mut::<starboard::input::CursorCell>() = cell;
    }

    fn sent(app: &App) -> Vec<starboard::canvas::DrawOp> {
        app.world()
            .resource::<room_sync::PendingOps<starboard::canvas::Canvas>>()
            .ops
            .clone()
    }

    #[test]
    fn mouse_press_sends_one_stamp() {
        let mut app = app_with_pointer();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        set_cursor(
            &mut app,
            Some(starboard::canvas::CellPos { x: 100, y: 100 }),
        );
        app.update();

        let ops = sent(&app);
        assert_eq!(ops.len(), 1);
        let stamp = starboard::canvas::CellPos { x: 100, y: 100 };
        assert_eq!(ops.first().map(|op| (op.from, op.to)), Some((stamp, stamp)));
        assert!(matches!(
            ops.first().map(|op| op.action),
            Some(starboard::canvas::DrawAction::Paint(_))
        ));
    }

    #[test]
    fn dragging_sends_the_segment_and_release_clears_the_tail() {
        let mut app = app_with_pointer();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        set_cursor(
            &mut app,
            Some(starboard::canvas::CellPos { x: 100, y: 100 }),
        );
        app.update();
        set_cursor(
            &mut app,
            Some(starboard::canvas::CellPos { x: 200, y: 100 }),
        );
        app.update();

        let ops = sent(&app);
        assert_eq!(ops.len(), 2);
        assert_eq!(
            ops.get(1).map(|op| (op.from, op.to)),
            Some((
                starboard::canvas::CellPos { x: 100, y: 100 },
                starboard::canvas::CellPos { x: 200, y: 100 }
            ))
        );

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
        set_cursor(
            &mut app,
            Some(starboard::canvas::CellPos { x: 100, y: 100 }),
        );
        app.update();

        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .reset_all();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Right);
        app.update();

        assert_eq!(
            sent(&app).get(1).map(|op| op.action),
            Some(starboard::canvas::DrawAction::Erase)
        );
    }

    #[test]
    fn a_still_cursor_sends_one_stamp_per_frame() {
        let mut app = app_with_pointer();
        app.world_mut()
            .resource_mut::<ButtonInput<MouseButton>>()
            .press(MouseButton::Left);
        set_cursor(
            &mut app,
            Some(starboard::canvas::CellPos { x: 300, y: 300 }),
        );
        app.update();
        app.update();
        assert_eq!(sent(&app).len(), 2);
    }
}
