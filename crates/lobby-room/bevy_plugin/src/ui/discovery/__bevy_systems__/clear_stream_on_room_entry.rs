use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::StreamedRoom;

pub fn clear_stream_on_room_entry(
    mut commands: MessageReader<core_discovery::Command>,
    mut streamed: ResMut<StreamedRoom>,
) {
    for command in commands.read() {
        if matches!(
            command,
            core_discovery::Command::Join(_) | core_discovery::Command::Create(_)
        ) {
            **streamed = None;
        }
    }
}
