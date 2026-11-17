use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::window::{CursorIcon, PrimaryWindow, SystemCursorIcon};

pub fn show_text_cursor_on_room_name_input(
    input: Single<&Hovered, With<ui::discovery::RoomNameInput>>,
    mut windows: Query<&mut CursorIcon, With<PrimaryWindow>>,
) {
    let is_hovered = input.into_inner().get();
    for mut cursor in &mut windows {
        let wanted = if is_hovered {
            CursorIcon::System(SystemCursorIcon::Text)
        } else {
            CursorIcon::default()
        };
        if *cursor != wanted {
            *cursor = wanted;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;
    use bevy::window::{CursorIcon, PrimaryWindow, SystemCursorIcon};

    use super::show_text_cursor_on_room_name_input;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow, CursorIcon::default()))
            .id();
        let input = app
            .world_mut()
            .spawn((ui::discovery::RoomNameInput, Hovered(true)))
            .id();
        app.add_systems(Update, show_text_cursor_on_room_name_input);

        app.update();
        assert_eq!(
            *app.world().get::<CursorIcon>(window).expect("cursor"),
            CursorIcon::System(SystemCursorIcon::Text)
        );

        app.world_mut().entity_mut(input).insert(Hovered(false));
        app.update();
        assert_eq!(
            *app.world().get::<CursorIcon>(window).expect("cursor"),
            CursorIcon::default()
        );
    }
}
