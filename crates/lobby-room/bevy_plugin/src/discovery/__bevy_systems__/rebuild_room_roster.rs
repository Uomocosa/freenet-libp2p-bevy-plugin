use bevy::prelude::*;

use crate::discovery;
use crate::p2p;

pub fn rebuild_room_roster(
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    ready: Option<Res<p2p::ReadyChannel>>,
    mut roster: ResMut<discovery::RoomRoster>,
) {
    let is_swarm_ready = ready.is_some_and(|ready| ready.ready_rx.borrow().is_some());
    *roster = state
        .map(|state| state.into_inner().room_roster(is_swarm_ready))
        .unwrap_or_default();
}
