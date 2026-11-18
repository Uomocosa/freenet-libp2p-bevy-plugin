use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{CloseStream, RoomStreamCloseButton};

pub fn close_stream_on_press(
    presses: Query<(), (Added<Pressed>, With<RoomStreamCloseButton>)>,
    mut commands: MessageWriter<CloseStream>,
) {
    if presses.is_empty() {
        return;
    }
    commands.write(CloseStream);
}
