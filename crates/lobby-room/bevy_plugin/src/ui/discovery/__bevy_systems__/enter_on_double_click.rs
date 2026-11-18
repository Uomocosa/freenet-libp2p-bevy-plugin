use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{DOUBLE_CLICK_SECS, RoomButton};

pub fn enter_on_double_click(
    buttons: Query<&RoomButton, Added<Pressed>>,
    rows: Option<Res<core_discovery::LobbyRows>>,
    time: Res<Time>,
    mut last_press: Local<Option<(core_discovery::RoomName, f32)>>,
    mut commands: MessageWriter<core_discovery::Command>,
) {
    let rows = rows.map(Res::into_inner);
    let now = time.into_inner().elapsed_secs();
    for button in &buttons {
        let room: core_discovery::RoomName = (**button).clone();
        let is_joinable = rows.is_some_and(|rows| {
            rows.iter()
                .find(|row| row.name == room)
                .is_some_and(|row| row.occupancy != core_discovery::RoomOccupancy::Full)
        });
        if !is_joinable {
            *last_press = None;
            continue;
        }
        let is_double = last_press
            .as_ref()
            .is_some_and(|(previous, at)| *previous == room && now - *at <= DOUBLE_CLICK_SECS);
        if is_double {
            commands.write(core_discovery::Command::Join(room));
            *last_press = None;
        } else {
            *last_press = Some((room, now));
        }
    }
}
