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
    pub(crate) fn perform<Replica: room_sync::RoomState>(
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
    pub(crate) fn snapshot_frame<Replica: room_sync::RoomState>(
        room: &Replica,
        room_name: identity::RoomName,
        part: u32,
    ) -> p2p::SnapshotResponse {
    }
}

#[rustfmt::skip]
impl<Op> SyncState<Op> {
    pub(crate) fn reset_after_leaving(&mut self) { *self = Self(room_sync::state_machine::ReplicaState { slice_anti_entropy: room_sync::state_machine::SliceAntiEntropyState { max_fetches_per_peer: self.slice_anti_entropy.max_fetches_per_peer, ..Default::default() }, kademlia_history: room_sync::state_machine::KademliaHistoryState { is_enabled: self.kademlia_history.is_enabled, max_fetch_retries: self.kademlia_history.max_fetch_retries, per_room_bytes: self.kademlia_history.per_room_bytes, record_ttl: self.kademlia_history.record_ttl, ..Default::default() }, ..Default::default() }); }
    pub(crate) fn libp2p_event_to_input<Replica: room_sync::RoomState>(
        event: &p2p::Libp2pEvent,
        state: &SyncState<Replica::Op>,
        clock: &mut room_sync::LamportClock,
        game: &identity::GameId,
        replica: &Replica,
    ) -> Option<room_sync::state_machine::Input<Replica::Op>> {
        match event {
            p2p::Libp2pEvent::Gossip { topic, from, data } => {
                let expected = state
                    .room
                    .as_ref()
                    .map(|room| identity::Topic::from_room_sync(game, room));
                if expected.as_ref() != Some(topic) {
                    return None;
                }
                let message =
                    bincode::deserialize::<room_sync::ReplicaSyncTopicMessage<Replica::Op>>(data)
                        .ok()?;
                match message {
                    room_sync::ReplicaSyncTopicMessage::ApplyThisOp { room, meta, op } => {
                        clock.observe(meta.lamport);
                        Some(room_sync::state_machine::Input::Remote { room, meta, op })
                    }
                    room_sync::ReplicaSyncTopicMessage::CompareSliceContentHashes {
                        hashes, ..
                    } => {
                        tracing::debug!(target: "room_sync", "anti-entropy compare from={from} remote_slices={}", hashes.len());
                        Some(room_sync::state_machine::Input::RemoteSliceContentHashes {
                            from: from.clone(),
                            local: replica.slice_content_hashes(),
                            remote: hashes,
                        })
                    }
                }
            }
            p2p::Libp2pEvent::SnapshotPart {
                room,
                part,
                total,
                data,
                ..
            } => Some(
                room_sync::state_machine::Input::SnapshotPartReceivedFromRoomMember {
                    room: room.clone(),
                    part: *part,
                    total: *total,
                    data: data.clone(),
                },
            ),
            p2p::Libp2pEvent::SnapshotMissing { .. } => {
                Some(room_sync::state_machine::Input::SnapshotMissingFromRoomMember)
            }
            p2p::Libp2pEvent::RoomSyncRequestFailed { from, request } => {
                match request {
                    p2p::SnapshotRequest::Fetch { .. } => {
                        Some(room_sync::state_machine::Input::RequestToRoomMemberFailed { from: from.clone() })
                    }
                    p2p::SnapshotRequest::FetchReplicaSlice { index, .. } => {
                        Some(room_sync::state_machine::Input::ReplicaSliceRequestFailed { from: from.clone(), index: *index })
                    }
                }
            }
            p2p::Libp2pEvent::HistoryRecordFound { id, value } => {
                Some(room_sync::state_machine::Input::HistoryRecordFound {
                    id: id.clone(),
                    value: value.clone(),
                })
            }
            p2p::Libp2pEvent::HistoryRecordNotFound { id } => {
                Some(room_sync::state_machine::Input::HistoryRecordNotFound { id: id.clone() })
            }
            p2p::Libp2pEvent::ReplicaSliceRequest {
                id, room, index, ..
            } => Some(room_sync::state_machine::Input::ReplicaSliceRequested {
                id: *id,
                room: room.clone(),
                index: *index,
            }),
            p2p::Libp2pEvent::ReplicaSlice {
                from,
                room,
                index,
                data,
            } => {
                tracing::debug!(target: "room_sync", "anti-entropy slice from={from} index={} bytes={}", **index, data.len());
                Some(room_sync::state_machine::Input::ReplicaSliceReceived {
                    from: from.clone(),
                    room: room.clone(),
                    index: *index,
                    data: data.clone(),
                })
            }
            p2p::Libp2pEvent::SnapshotRequest { id, room, part, .. } => Some(
                room_sync::state_machine::Input::SnapshotPartRequestedByRoomMember {
                    id: *id,
                    room: room.clone(),
                    part: *part,
                },
            ),
            _ => None,
        }
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
