use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::EnterRoomButton;

pub fn handle_join(
    buttons: Query<&EnterRoomButton, Added<Pressed>>,
    mut commands: MessageWriter<core_discovery::Command>,
) {
    for button in &buttons {
        let room: &core_discovery::RoomName = button;
        tracing::info!(target: "discovery", "ui join room={}", room.as_str());
        commands.write(core_discovery::Command::Join(room.clone()));
    }
}
