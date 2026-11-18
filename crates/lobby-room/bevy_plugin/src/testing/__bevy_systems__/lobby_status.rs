use bevy::prelude::*;
use bevy::remote::{BrpError, BrpResult};
use serde_json::Value;

use crate::discovery;
use crate::testing;
use crate::ui;

pub fn lobby_status(
    In(_params): In<Option<Value>>,
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    join_error: Option<Res<ui::discovery::JoinError>>,
) -> BrpResult {
    let mut status = state.map_or_else(testing::LobbyStatus::default, |state| {
        testing::LobbyStatus {
            rooms: state
                .lobby
                .keys()
                .map(|room| room.as_str().to_owned())
                .collect(),
            joined_room: state
                .room
                .as_ref()
                .map(|room| room.name.as_str().to_owned()),
            other_members: state.room.as_ref().map_or(0, |room| room.members.len()),
            connected: state.connected.len(),
            lobby_fetched: state.is_lobby_fetched,
            join_error: None,
        }
    });
    status.join_error = join_error.and_then(|error| (**error).clone());
    serde_json::to_value(status).map_err(BrpError::internal)
}
