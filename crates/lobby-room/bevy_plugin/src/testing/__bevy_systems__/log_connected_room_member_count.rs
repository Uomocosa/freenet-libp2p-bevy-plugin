use bevy::prelude::*;

use crate::discovery;
use crate::testing;

pub fn log_connected_room_member_count(
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    mut last_logged: Local<Option<testing::LoggedRoomMemberCount>>,
) {
    let Some(state) = state else {
        return;
    };
    if !state.is_changed() {
        return;
    }
    let Some(room) = state.room.as_ref() else {
        return;
    };
    let now = testing::LoggedRoomMemberCount {
        room: room.name.clone(),
        connected: state.connected_members().count(),
    };
    if last_logged.as_ref() == Some(&now) {
        return;
    }
    tracing::info!(
        "lobby mesh room={} connected={}",
        now.room.as_str(),
        now.connected
    );
    *last_logged = Some(now);
}
