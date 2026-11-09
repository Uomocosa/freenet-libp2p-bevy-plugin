use bevy::prelude::*;
use bevy::window::PrimaryWindow;

use crate::starboard;

pub fn track_cursor(
    windows: Query<&Window, With<PrimaryWindow>>,
    view: Res<starboard::render::CanvasView>,
    mut cursor: ResMut<starboard::input::CursorCell>,
) {
    let view = view.into_inner();
    let next = windows
        .single()
        .ok()
        .and_then(Window::cursor_position)
        .and_then(|position| starboard::input::cell_under_cursor(position, view));
    if **cursor != next {
        **cursor = next;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;

    use super::track_cursor;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(starboard::render::CanvasView::fit(Vec2::splat(1024.0)));
        app.init_resource::<starboard::input::CursorCell>();
        app.world_mut().spawn((
            PrimaryWindow,
            Window {
                resolution: (1024, 1024).into(),
                ..default()
            },
        ));
        app.add_systems(Update, track_cursor);

        app.update();
        assert_eq!(
            **app.world().resource::<starboard::input::CursorCell>(),
            None
        );

        let mut windows = app
            .world_mut()
            .query_filtered::<&mut Window, With<PrimaryWindow>>();
        let mut window = windows
            .single_mut(app.world_mut())
            .expect("one primary window");
        window.set_cursor_position(Some(Vec2::new(300.5, 200.5)));
        app.update();
        assert_eq!(
            **app.world().resource::<starboard::input::CursorCell>(),
            Some(starboard::canvas::CellPos { x: 300, y: 200 })
        );
    }
}
