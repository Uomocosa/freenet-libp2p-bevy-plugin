use bevy::prelude::*;

use crate::peers;
use crate::replica_sync;
use crate::testing;

pub fn capture_zone_inputs<Op: Send + Sync + 'static>(
    mut state: ResMut<replica_sync::bevy_systems::SyncState<Op>>,
    game: Res<replica_sync::SyncGame>,
    me_zones: Query<&peers::InZone, With<peers::Me>>,
    zone_details: Query<replica_sync::ZoneDetails<'_>>,
    member_peers: Query<(&peers::Peer, Has<peers::Connected>, Has<peers::Me>)>,
    mut captured: ResMut<testing::CapturedOutputs<Op>>,
) {
    let game = game.into_inner();
    let (commands, inputs) = replica_sync::bevy_systems::SyncState::<Op>::zone_inputs_from_entities(
        &mut state,
        game,
        &me_zones,
        &zone_details,
        &member_peers,
    );
    captured.commands = commands;
    captured.inputs = inputs;
}
