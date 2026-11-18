use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Has;
use bevy::prelude::MessageWriter;
use bevy::prelude::Query;
use bevy::prelude::Resource;
use bevy::prelude::With;
use derive_more::{Deref, DerefMut};

use crate::identity;
use crate::p2p;
use crate::peers;
use crate::replica_sync;

#[derive(Resource, Deref, DerefMut)]
pub struct SyncState<Op>(pub replica_sync::state_machine::ReplicaState<Op>);

impl<Op> Default for SyncState<Op> {
    fn default() -> Self {
        Self(replica_sync::state_machine::ReplicaState::default())
    }
}

#[atomic_delegates]
impl<Op> SyncState<Op> {
    pub(crate) fn zone_inputs_from_entities(
        state: &mut Self,
        game: &identity::GameId,
        me_zones: &Query<&peers::InZone, With<peers::Me>>,
        zone_details: &Query<replica_sync::ZoneDetails<'_>>,
        member_peers: &Query<(&peers::Peer, Has<peers::Connected>, Has<peers::Me>)>,
    ) -> (
        Vec<p2p::Libp2pCommand>,
        Vec<replica_sync::state_machine::Input<Op>>,
    ) {
    }
    pub(crate) fn perform<Replica: replica_sync::traits::ReplicatedState>(
        output: replica_sync::state_machine::Output<Replica::Op>,
        zone: &mut Replica,
        clock: replica_sync::LamportClock,
        per_zone_bytes: p2p::HistoryByteCount,
        record_ttl: Duration,
        pending: &mut Vec<p2p::Libp2pCommand>,
        failures: &mut Option<MessageWriter<replica_sync::JoinFailure>>,
        game: &identity::GameId,
    ) {
    }
    pub(crate) fn reset_after_leaving(&mut self) {}
    pub(crate) fn snapshot_frame<Replica: replica_sync::traits::ReplicatedState>(
        zone: &Replica,
        zone_name: identity::ZoneName,
        part: u32,
    ) -> p2p::SnapshotResponse {
    }
    pub(crate) fn libp2p_event_to_input<Replica: replica_sync::traits::ReplicatedState>(
        event: &p2p::Libp2pEvent,
        state: &SyncState<Replica::Op>,
        clock: &mut replica_sync::LamportClock,
        game: &identity::GameId,
        replica: &Replica,
    ) -> Option<replica_sync::state_machine::Input<Replica::Op>> {
    }
    pub(crate) fn topic_message_to_input<Replica: replica_sync::traits::ReplicatedState>(
        message: replica_sync::ReplicaSyncTopicMessage<Replica::Op>,
        from: &identity::PeerId,
        clock: &mut replica_sync::LamportClock,
        replica: &Replica,
    ) -> Option<replica_sync::state_machine::Input<Replica::Op>> {
    }
}
