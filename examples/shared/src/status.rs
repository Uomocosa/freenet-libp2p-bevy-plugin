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

pub fn log_tick(time: Res<Time>, state: Option<Res<DiscoveryState>>, mut accumulator: Local<f32>) {
    *accumulator += time.delta_secs();
    if *accumulator < 1.0 {
        return;
    }
    *accumulator = 0.0;
    let rooms: Vec<&str> = state.as_deref().map_or_else(Vec::new, |state| {
        state.lobby.keys().map(|name| name.as_str()).collect()
    });
    let rooms = if rooms.is_empty() {
        "none".to_string()
    } else {
        rooms.join(",")
    };
    let room = state
        .as_deref()
        .and_then(|state| state.room.as_ref())
        .map_or_else(|| "none".to_string(), |room| room.name.as_str().to_string());
    let members = state
        .as_deref()
        .and_then(|state| state.room.as_ref())
        .map_or(0, |room| room.members.len());
    let connected = state
        .as_deref()
        .map_or(0, |state| state.connected_members().count());
    let freenet_peers = state
        .as_deref()
        .map_or(0, |state| state.freenet_peers.len());
    tracing::info!(
        "lobby tick rooms={rooms} room={room} members={members} connected={connected} freenet_peers={freenet_peers}"
    );
}

// no test_usage necessary: log-only systems, covered by the e2e log parser
