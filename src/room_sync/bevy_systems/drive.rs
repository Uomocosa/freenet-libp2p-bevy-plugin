use bevy::prelude::*;

use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::room_sync;

pub fn drive<S: room_sync::RoomState>(
    room_events: Option<MessageReader<discovery::Event>>,
    snapshot: Option<Res<discovery::Snapshot>>,
    net_events: Res<room_sync::bevy_systems::NetEvents>,
    game: Res<room_sync::RoomSyncGame>,
    ready: Res<p2p::ReadyChannel>,
    mut clock: ResMut<room_sync::LamportClock>,
    mut state: ResMut<room_sync::bevy_systems::SyncState<S::Op>>,
    mut room: ResMut<S>,
    mut pending: ResMut<p2p::PendingNetCommands>,
) {
    let snapshot = snapshot.map(Res::into_inner);
    let net_events = net_events.into_inner();
    let game = game.into_inner();
    let ready = ready.into_inner();
    let own = ready.ready_rx.borrow().clone().map(|peer| peer.id);
    let members: Vec<identity::PeerId> = snapshot
        .as_ref()
        .and_then(|snapshot| snapshot.room.as_ref())
        .map_or_else(Vec::new, |joined| {
            joined
                .members
                .iter()
                .filter(|(peer, member)| {
                    member.status == discovery::LinkStatus::Connected && Some(*peer) != own.as_ref()
                })
                .map(|(peer, _)| peer.clone())
                .collect()
        });
    let mut inputs: Vec<room_sync::state_machine::Input<S::Op>> = Vec::new();
    if let Some(mut room_events) = room_events {
        for event in room_events.read() {
            match event {
                discovery::Event::Joined(room_name) => {
                    pending.push(p2p::NetCommand::Subscribe {
                        topic: room_sync::sync_topic(game, room_name),
                    });
                    inputs.push(room_sync::state_machine::Input::Joined {
                        room: room_name.clone(),
                        members: members.clone(),
                    });
                }
                discovery::Event::Left(room_name) => {
                    pending.push(p2p::NetCommand::Unsubscribe {
                        topic: room_sync::sync_topic(game, room_name),
                    });
                }
                discovery::Event::MembersChanged => {
                    inputs.push(room_sync::state_machine::Input::Members {
                        members: members.clone(),
                    });
                }
                _ => {}
            }
        }
    }
    for event in net_events.drain() {
        if let Some(input) = net_event_to_input::<S>(&event, &state, &mut clock, game) {
            inputs.push(input);
        }
    }
    for input in inputs {
        for output in room_sync::state_machine::update(&mut state, input) {
            perform(output, &mut *room, &mut pending);
        }
    }
}

// needed helper: one net event as a reducer input, if it concerns this room
fn net_event_to_input<S: room_sync::RoomState>(
    event: &p2p::NetEvent,
    state: &room_sync::bevy_systems::SyncState<S::Op>,
    clock: &mut room_sync::LamportClock,
    game: &identity::GameId,
) -> Option<room_sync::state_machine::Input<S::Op>> {
    match event {
        p2p::NetEvent::Gossip { topic, data, .. } => {
            let expected = state
                .room
                .as_ref()
                .map(|room| room_sync::sync_topic(game, room));
            if expected.as_ref() != Some(topic) {
                return None;
            }
            let frame = bincode::deserialize::<room_sync::SyncFrame<S::Op>>(data).ok()?;
            clock.observe(frame.meta.lamport);
            Some(room_sync::state_machine::Input::Remote {
                room: frame.room,
                meta: frame.meta,
                op: frame.op,
            })
        }
        p2p::NetEvent::SnapshotPart {
            room,
            part,
            total,
            data,
            ..
        } => Some(room_sync::state_machine::Input::Snapshot {
            room: room.clone(),
            part: *part,
            total: *total,
            data: data.clone(),
        }),
        p2p::NetEvent::SnapshotMissing { .. } => Some(room_sync::state_machine::Input::Tick),
        p2p::NetEvent::SnapshotRequest { id, room, part, .. } => {
            Some(room_sync::state_machine::Input::Fetch {
                id: *id,
                room: room.clone(),
                part: *part,
            })
        }
        _ => None,
    }
}

// needed helper: executes one reducer output
fn perform<S: room_sync::RoomState>(
    output: room_sync::state_machine::Output<S::Op>,
    room: &mut S,
    pending: &mut p2p::PendingNetCommands,
) {
    match output {
        room_sync::state_machine::Output::Fetch {
            peer,
            room: room_name,
            part,
        } => {
            pending.push(p2p::NetCommand::FetchSnapshot {
                peer_id: peer,
                room: room_name,
                part,
            });
        }
        room_sync::state_machine::Output::Snapshot {
            id,
            room: room_name,
            part,
        } => {
            pending.push(p2p::NetCommand::RespondSnapshot {
                id,
                frame: snapshot_frame(room, room_name, part),
            });
        }
        room_sync::state_machine::Output::Merge { bytes } => {
            if let Ok(snapshot) = bincode::deserialize::<S::Snapshot>(&bytes) {
                room.merge(snapshot);
            }
        }
        room_sync::state_machine::Output::Apply { meta, op } => room.apply(meta, &op),
    }
}

// needed helper: one snapshot part, or Missing when the index is out of range
fn snapshot_frame<S: room_sync::RoomState>(
    room: &S,
    room_name: identity::RoomName,
    part: u32,
) -> p2p::RoomSyncFrame {
    let Ok(bytes) = bincode::serialize(&room.snapshot()) else {
        return p2p::RoomSyncFrame::Missing { room: room_name };
    };
    let parts = room_sync::split_parts(&bytes, room_sync::PART_BYTES);
    let total = u32::try_from(parts.len()).unwrap_or(u32::MAX);
    let data = usize::try_from(part)
        .ok()
        .and_then(|index| parts.get(index).cloned());
    match data {
        Some(data) => p2p::RoomSyncFrame::Part {
            room: room_name,
            part,
            total,
            data,
        },
        None => p2p::RoomSyncFrame::Missing { room: room_name },
    }
}

// no test_usage necessary
