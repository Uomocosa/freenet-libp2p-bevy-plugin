use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::SelectedRoom;

pub fn drop_stale_room_selection(
    rows: Option<Res<core_discovery::LobbyRows>>,
    mut selection: ResMut<SelectedRoom>,
) {
    let Some(room) = (**selection).clone() else {
        return;
    };
    let rows = rows.map(Res::into_inner);
    let is_present = rows.is_some_and(|rows| rows.iter().any(|row| row.name == room));
    if !is_present {
        **selection = None;
    }
}
