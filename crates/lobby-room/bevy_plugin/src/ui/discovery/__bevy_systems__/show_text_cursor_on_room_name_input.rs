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
