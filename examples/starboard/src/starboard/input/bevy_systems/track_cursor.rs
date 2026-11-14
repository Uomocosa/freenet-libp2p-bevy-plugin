use bevy::picking::pointer::{PointerId, PointerLocation};
use bevy::prelude::*;
use bevy::window::{CursorLeft, CursorMoved};

use crate::starboard;

pub fn track_cursor(
    pointers: Query<(&PointerId, &PointerLocation)>,
    view: Res<starboard::render::CanvasView>,
    mut moved: MessageReader<CursorMoved>,
    mut left: MessageReader<CursorLeft>,
    mut outside: Local<bool>,
    mut cursor: ResMut<starboard::input::CursorCell>,
    mut point: ResMut<starboard::input::CursorPoint>,
) {
    let view = view.into_inner();
    if left.read().last().is_some() {
        *outside = true;
    }
    if moved.read().last().is_some() {
        *outside = false;
    }
    let screen = if *outside {
        None
    } else {
        pointers
            .iter()
            .find(|(id, _)| id.is_mouse())
            .and_then(|(_, location)| location.location())
            .map(|location| location.position)
    };
    let next = screen.and_then(|position| starboard::input::cell_under_cursor(position, view));
    if **cursor != next {
        **cursor = next;
    }
    let next_point =
        screen.and_then(|position| starboard::input::sub_cell_under_cursor(position, view));
    if **point != next_point {
        **point = next_point;
    }
}

#[cfg(test)]
mod tests {
    use bevy::camera::{ManualTextureViewHandle, NormalizedRenderTarget};
    use bevy::picking::pointer::{Location, PointerId, PointerLocation, PointerMap};
    use bevy::prelude::*;
    use bevy::window::{CursorLeft, CursorMoved};

    use super::track_cursor;
    use crate::starboard;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(starboard::render::CanvasView::fit(Vec2::splat(1024.0)));
        app.init_resource::<starboard::input::CursorCell>();
        app.init_resource::<starboard::input::CursorPoint>();
        app.init_resource::<PointerMap>();
        app.world_mut()
            .spawn((PointerId::Mouse, PointerLocation::default()));
        app.add_message::<CursorMoved>();
        app.add_message::<CursorLeft>();
        app.add_systems(Update, track_cursor);

        app.update();
        assert_eq!(
            **app.world().resource::<starboard::input::CursorCell>(),
            None
        );

        let mut pointers = app
            .world_mut()
            .query_filtered::<&mut PointerLocation, With<PointerId>>();
        let mut location = pointers.single_mut(app.world_mut()).expect("one mouse");
        location.location = Some(Location {
            target: NormalizedRenderTarget::TextureView(ManualTextureViewHandle(0)),
            position: Vec2::new(300.5, 200.5),
        });
        app.update();
        assert_eq!(
            **app.world().resource::<starboard::input::CursorCell>(),
            Some(starboard::canvas::CellPos { x: 300, y: 200 })
        );
    }

    #[test]
    fn test_leaving_the_window_clears_the_cursor_until_it_moves_again() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(starboard::render::CanvasView::fit(Vec2::splat(1024.0)));
        app.init_resource::<starboard::input::CursorCell>();
        app.init_resource::<starboard::input::CursorPoint>();
        app.init_resource::<PointerMap>();
        app.add_message::<CursorMoved>();
        app.add_message::<CursorLeft>();
        let window = app.world_mut().spawn_empty().id();
        app.world_mut().spawn((
            PointerId::Mouse,
            PointerLocation::new(Location {
                target: NormalizedRenderTarget::TextureView(ManualTextureViewHandle(0)),
                position: Vec2::new(300.5, 200.5),
            }),
        ));
        app.add_systems(Update, track_cursor);
        app.update();
        assert_eq!(
            **app.world().resource::<starboard::input::CursorCell>(),
            Some(starboard::canvas::CellPos { x: 300, y: 200 })
        );
        app.world_mut().write_message(CursorLeft { window });
        app.update();
        assert_eq!(
            **app.world().resource::<starboard::input::CursorCell>(),
            None
        );
        app.world_mut().write_message(CursorMoved {
            window,
            position: Vec2::new(300.5, 200.5),
            delta: None,
        });
        app.update();
        assert_eq!(
            **app.world().resource::<starboard::input::CursorCell>(),
            Some(starboard::canvas::CellPos { x: 300, y: 200 })
        );
    }
}
