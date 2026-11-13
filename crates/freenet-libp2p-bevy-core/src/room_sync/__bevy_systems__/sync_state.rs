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
use crate::room_sync;

#[derive(Resource, Deref, DerefMut)]
pub struct SyncState<Op>(pub room_sync::state_machine::ReplicaState<Op>);

impl<Op> Default for SyncState<Op> {
    fn default() -> Self {
        Self(room_sync::state_machine::ReplicaState::default())
    }
}

#[atomic_delegates]
impl<Op> SyncState<Op> {
    pub(crate) fn room_inputs_from_entities(
        state: &mut Self,
        game: &identity::GameId,
        me_rooms: &Query<&peers::InRoom, With<peers::Me>>,
        room_details: &Query<room_sync::RoomDetails<'_>>,
        member_peers: &Query<(&peers::Peer, Has<peers::Connected>, Has<peers::Me>)>,
    ) -> (
        Vec<p2p::Libp2pCommand>,
        Vec<room_sync::state_machine::Input<Op>>,
    ) {
    }
    pub(crate) fn perform<Replica: room_sync::traits::ReplicatedState>(
        output: room_sync::state_machine::Output<Replica::Op>,
        room: &mut Replica,
        clock: room_sync::LamportClock,
        per_room_bytes: p2p::HistoryByteCount,
        record_ttl: Duration,
        pending: &mut p2p::PendingLibp2pCommands,
        failures: &mut Option<MessageWriter<room_sync::JoinFailure>>,
        game: &identity::GameId,
    ) {
    }
    pub(crate) fn reset_after_leaving(&mut self) {}
    pub(crate) fn snapshot_frame<Replica: room_sync::traits::ReplicatedState>(
        room: &Replica,
        room_name: identity::RoomName,
        part: u32,
    ) -> p2p::SnapshotResponse {
    }
    pub(crate) fn libp2p_event_to_input<Replica: room_sync::traits::ReplicatedState>(
        event: &p2p::Libp2pEvent,
        state: &SyncState<Replica::Op>,
        clock: &mut room_sync::LamportClock,
        game: &identity::GameId,
        replica: &Replica,
    ) -> Option<room_sync::state_machine::Input<Replica::Op>> {
    }
}

#[cfg(test)]
mod tests {
    use super::SyncState;
    use crate::identity;
    use crate::room_sync;

    #[test]
    fn test_usage() {
        let mut state = SyncState::<u8>(room_sync::state_machine::ReplicaState {
            room: Some(identity::RoomName::from("room")),
            slice_anti_entropy: room_sync::state_machine::SliceAntiEntropyState {
                max_fetches_per_peer: 3,
                ..Default::default()
            },
            kademlia_history: room_sync::state_machine::KademliaHistoryState {
                is_enabled: true,
                ..Default::default()
            },
            ..Default::default()
        });
        state.reset_after_leaving();
        assert_eq!(state.room, None);
        assert_eq!(state.phase, room_sync::state_machine::Phase::Idle);
        assert_eq!(state.slice_anti_entropy.max_fetches_per_peer, 3);
        assert!(state.kademlia_history.is_enabled);
    }
}
