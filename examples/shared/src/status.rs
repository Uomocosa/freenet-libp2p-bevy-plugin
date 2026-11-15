#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::identity;
use freenet_libp2p_bevy_plugin::lobby_room::discovery;

#[derive(Resource, Deref)]
pub struct LocalUser(pub identity::Username);

pub fn log_mesh(state: Option<Res<DiscoveryState>>, mut last: Local<Option<(String, usize)>>) {
    let Some(state) = state.as_ref() else {
        return;
    };
    if !state.is_changed() {
        return;
    }
    let Some(room) = state.room.as_ref() else {
        return;
    };
    let connected = state.connected_members().count();
    let now = (room.name.as_str().to_string(), connected);
    if last.as_ref() == Some(&now) {
        return;
    }
    tracing::info!(
        "lobby mesh room={} connected={connected}",
        room.name.as_str()
    );
    *last = Some(now);
}

// no test_usage necessary: log-only system, its marker is read by the e2e log parser
