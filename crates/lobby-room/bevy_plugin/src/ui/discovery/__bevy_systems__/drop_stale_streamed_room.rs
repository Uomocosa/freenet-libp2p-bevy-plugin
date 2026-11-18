use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::StreamedRoom;

pub fn drop_stale_streamed_room(
    rows: Option<Res<core_discovery::LobbyRows>>,
    mut streamed: ResMut<StreamedRoom>,
) {
    let Some(room) = (**streamed).clone() else {
        return;
    };
    let rows = rows.map(Res::into_inner);
    let is_present = rows.is_some_and(|rows| rows.iter().any(|row| row.name == room));
    if !is_present {
        **streamed = None;
    }
}
