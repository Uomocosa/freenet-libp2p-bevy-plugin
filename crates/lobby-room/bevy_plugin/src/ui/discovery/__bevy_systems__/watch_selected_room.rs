use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::{SelectedRoom, StreamedRoom};

pub fn watch_selected_room(
    selection: Res<SelectedRoom>,
    streamed: Res<StreamedRoom>,
    mut commands: MessageWriter<core_discovery::Command>,
    mut watched_room: Local<Option<core_discovery::RoomName>>,
    mut watched_streamed_room: Local<Option<core_discovery::RoomName>>,
) {
    let selected_room = (**selection.into_inner()).clone();
    let streamed_room = (**streamed.into_inner()).clone();
    if *watched_streamed_room != streamed_room {
        (*watched_streamed_room).clone_from(&streamed_room);
        if let Some(room) = &streamed_room {
            commands.write(core_discovery::Command::Watch(room.clone()));
        }
    }
    if *watched_room != selected_room {
        (*watched_room).clone_from(&selected_room);
        if selected_room.as_ref() != streamed_room.as_ref()
            && let Some(room) = &selected_room
        {
            commands.write(core_discovery::Command::Watch(room.clone()));
        }
    }
}
