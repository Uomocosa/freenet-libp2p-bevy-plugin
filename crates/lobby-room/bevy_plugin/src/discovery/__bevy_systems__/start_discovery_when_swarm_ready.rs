use bevy::prelude::*;

use crate::discovery;
use crate::p2p;

pub fn start_discovery_when_swarm_ready(
    mut commands: Commands,
    ready: Option<Res<p2p::ReadyChannel>>,
    config: Option<Res<discovery::Config>>,
) {
    let Some(ready) = ready.map(Res::into_inner) else {
        return;
    };
    let Some(peer) = ready.ready_rx.borrow().clone() else {
        return;
    };
    let Some(config) = config.map(Res::into_inner) else {
        return;
    };
    let state = discovery::state_machine::DiscoveryState::from_ready_peer(peer, config);
    commands.insert_resource(state);
}
