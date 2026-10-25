use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::{ResMut, Resource};
use derive_more::{Deref, DerefMut};

use crate::discovery;
use crate::identity;
use crate::p2p;
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
    pub(crate) fn perform<Replica: room_sync::RoomState>(
        output: room_sync::state_machine::Output<Replica::Op>,
        room: &mut Replica,
        clock: room_sync::LamportClock,
        per_room_bytes: p2p::HistoryByteCount,
        record_ttl: Duration,
        pending: &mut p2p::PendingNetCommands,
        command_sender: Option<&discovery::CommandSender>,
        join_error: &mut Option<ResMut<discovery::ui::JoinError>>,
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
    pub(crate) fn net_event_to_input<Replica: room_sync::RoomState>(
        event: &p2p::NetEvent,
        state: &SyncState<Replica::Op>,
        clock: &mut room_sync::LamportClock,
        game: &identity::GameId,
        replica: &Replica,
    ) -> Option<room_sync::state_machine::Input<Replica::Op>> {
        match event {
            p2p::NetEvent::Gossip { topic, from, data } => {
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
                    } => Some(room_sync::state_machine::Input::RemoteSliceContentHashes {
                        from: from.clone(),
                        local: replica.slice_content_hashes(),
                        remote: hashes,
                    }),
                }
            }
            p2p::NetEvent::SnapshotPart {
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
            p2p::NetEvent::SnapshotMissing { .. } => Some(room_sync::state_machine::Input::Tick),
            p2p::NetEvent::SnapshotFailed { from } => {
                Some(room_sync::state_machine::Input::RequestToRoomMemberFailed { from: from.clone() })
            }
            p2p::NetEvent::HistoryRecordFound { id, value } => {
                Some(room_sync::state_machine::Input::HistoryRecordFound {
                    id: id.clone(),
                    value: value.clone(),
                })
            }
            p2p::NetEvent::HistoryRecordNotFound { id } => {
                Some(room_sync::state_machine::Input::HistoryRecordNotFound { id: id.clone() })
            }
            p2p::NetEvent::ReplicaSliceRequest {
                id, room, index, ..
            } => Some(room_sync::state_machine::Input::ReplicaSliceRequested {
                id: *id,
                room: room.clone(),
                index: *index,
            }),
            p2p::NetEvent::ReplicaSlice {
                from,
                room,
                index,
                data,
            } => Some(room_sync::state_machine::Input::ReplicaSliceReceived {
                from: from.clone(),
                room: room.clone(),
                index: *index,
                data: data.clone(),
            }),
            p2p::NetEvent::SnapshotRequest { id, room, part, .. } => Some(
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

// no test_usage necessary
