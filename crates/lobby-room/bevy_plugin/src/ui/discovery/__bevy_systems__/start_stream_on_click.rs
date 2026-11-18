use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{RoomButton, StreamedRoom};

pub fn start_stream_on_click(
    buttons: Query<&RoomButton, Added<Pressed>>,
    mut streamed: ResMut<StreamedRoom>,
) {
    for button in &buttons {
        **streamed = Some((**button).clone());
    }
}
