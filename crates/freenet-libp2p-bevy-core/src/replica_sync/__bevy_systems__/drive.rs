use bevy::prelude::*;

use crate::p2p;
use crate::peers;
use crate::replica_sync;

type ReplicaInputSources<'a, 's, Op> = (
    Res<'a, replica_sync::bevy_systems::Libp2pEvents>,
    MessageReader<'a, 's, replica_sync::ReplicaInputFromTimer<Op>>,
    Res<'a, Time<Real>>,
);

pub fn drive<Replica: replica_sync::traits::ReplicatedState>(
    mut failures: Option<MessageWriter<replica_sync::JoinFailure>>,
    (libp2p_events, mut timer_inputs, real_time): ReplicaInputSources<'_, '_, Replica::Op>,
    game: Res<replica_sync::SyncGame>,
    mut clock: ResMut<replica_sync::LamportClock>,
    mut state: ResMut<replica_sync::bevy_systems::SyncState<Replica::Op>>,
    mut zone: ResMut<Replica>,
    mut commands: MessageWriter<p2p::Libp2pCommand>,
    me_zones: Query<&peers::InZone, With<peers::Me>>,
    zone_details: Query<replica_sync::ZoneDetails<'_>>,
    member_peers: Query<(&peers::Peer, Has<peers::Connected>, Has<peers::Me>)>,
) {
    let libp2p_events = libp2p_events.into_inner();
    let game = game.into_inner();
    let (zone_commands, mut inputs) =
        replica_sync::bevy_systems::SyncState::<Replica::Op>::zone_inputs_from_entities(
            &mut state,
            game,
            &me_zones,
            &zone_details,
            &member_peers,
        );
    let mut pending = zone_commands;
    for event in libp2p_events.drain() {
        let input = replica_sync::bevy_systems::SyncState::<Replica::Op>::libp2p_event_to_input(
            &event, &state, &mut clock, game, &*zone,
        );
        inputs.extend(input);
    }
    for timer_input in timer_inputs.read() {
        inputs.push((**timer_input).clone());
    }
    let now = replica_sync::ElapsedSinceAppStart::from(real_time.elapsed());
    let per_zone_bytes = state.kademlia_history.per_zone_bytes;
    let record_ttl = state.kademlia_history.record_ttl;
    for input in inputs {
        for output in state.update(input, now) {
            if let replica_sync::state_machine::Output::FetchReplicaSlice { peer, index, .. } =
                &output
            {
                tracing::debug!(target: "replica_sync", "anti-entropy fetch peer={peer} index={}", **index);
            }
            replica_sync::bevy_systems::SyncState::<Replica::Op>::perform(
                output,
                &mut *zone,
                *clock,
                per_zone_bytes,
                record_ttl,
                &mut pending,
                &mut failures,
                game,
            );
        }
    }
    commands.write_batch(pending);
}
