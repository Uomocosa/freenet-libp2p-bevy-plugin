use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::JoinError;

pub fn record_join_error(
    mut events: MessageReader<core_discovery::Event>,
    mut join_error: ResMut<JoinError>,
) {
    for event in events.read() {
        match event {
            core_discovery::Event::RoomGone(room) => {
                **join_error = Some(format!(
                    "Room {} no longer exists (all players left)",
                    room.as_str()
                ));
            }
            core_discovery::Event::HistoryUnavailable(_room) => {
                **join_error = Some(String::from("Couldn't get room history"));
            }
            core_discovery::Event::Joined(_) => {
                **join_error = None;
            }
            _ => {}
        }
    }
}
