use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{RoomStreamFullscreenButton, StreamDisplay};

pub fn toggle_stream_fullscreen_on_press(
    presses: Query<(), (Added<Pressed>, With<RoomStreamFullscreenButton>)>,
    mut display: ResMut<StreamDisplay>,
) {
    if presses.is_empty() {
        return;
    }
    display.toggle();
}
