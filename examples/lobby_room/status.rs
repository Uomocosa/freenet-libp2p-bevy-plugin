#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{discovery, identity};

#[derive(Resource, Deref)]
pub struct LocalUser(pub identity::Username);

/// Event-driven `lobby mesh` marker: logs the live link count the moment it changes.
pub fn log_mesh(snapshot: Res<discovery::Snapshot>, mut last: Local<Option<(String, usize)>>) {
    if !snapshot.is_changed() {
        return;
    }
    let Some(room) = snapshot.room.as_ref() else {
        return;
    };
    let connected = room
        .members
        .values()
        .filter(|member| member.status == discovery::LinkStatus::Connected)
        .count();
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

pub fn log_tick(time: Res<Time>, snapshot: Res<discovery::Snapshot>, mut accumulator: Local<f32>) {
    *accumulator += time.delta_secs();
    if *accumulator < 1.0 {
        return;
    }
    *accumulator = 0.0;
    let rooms: Vec<&str> = snapshot.lobby.keys().map(|name| name.as_str()).collect();
    let rooms = if rooms.is_empty() {
        "none".to_string()
    } else {
        rooms.join(",")
    };
    let room = snapshot
        .room
        .as_ref()
        .map_or_else(|| "none".to_string(), |room| room.name.as_str().to_string());
    let members = snapshot.room.as_ref().map_or(0, |room| room.members.len());
    let connected = snapshot.room.as_ref().map_or(0, |room| {
        room.members
            .values()
            .filter(|member| member.status == discovery::LinkStatus::Connected)
            .count()
    });
    let freenet_peers = snapshot.freenet_peers.len();
    tracing::info!(
        "lobby tick rooms={rooms} room={room} members={members} connected={connected} freenet_peers={freenet_peers}"
    );
}

// no test_usage necessary: log-only systems, covered by the e2e log parser
