use bevy::prelude::*;

use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::room_sync;

pub fn drive<Replica: room_sync::RoomState>(
    room_events: Option<MessageReader<discovery::Event>>,
    command_sender: Option<Res<discovery::CommandSender>>,
    mut join_error: Option<ResMut<discovery::ui::JoinError>>,
    status: Option<Res<discovery::Status>>,
    ready: Option<Res<p2p::ReadyChannel>>,
    holder_heartbeat_intervals: Option<Res<discovery::HolderHeartbeatIntervals>>,
    net_events: Res<room_sync::bevy_systems::NetEvents>,
    game: Res<room_sync::SyncGame>,
    mut clock: ResMut<room_sync::LamportClock>,
    mut state: ResMut<room_sync::bevy_systems::SyncState<Replica::Op>>,
    mut room: ResMut<Replica>,
    mut pending: ResMut<p2p::PendingNetCommands>,
) {
    let status = status.map(Res::into_inner);
    let command_sender = command_sender.map(Res::into_inner);
    let net_events = net_events.into_inner();
    let game = game.into_inner();
    let self_peer_id = ready
        .and_then(|ready| ready.ready_rx.borrow().clone())
        .map(|peer| peer.id);
    let ttl = holder_heartbeat_intervals.map_or_else(
        || discovery::HolderHeartbeatIntervals::default().ttl,
        |intervals| intervals.ttl,
    );
    let members: Vec<identity::PeerId> = status
        .as_ref()
        .and_then(|status| status.room.as_ref())
        .map_or_else(Vec::new, |joined| {
            joined
                .members
                .iter()
                .filter(|(_, member)| member.status == discovery::LinkStatus::Connected)
                .map(|(peer, _)| peer.clone())
                .collect()
        });
    let mut inputs: Vec<room_sync::state_machine::Input<Replica::Op>> = Vec::new();
    if let Some(mut room_events) = room_events {
        for event in room_events.read() {
            match event {
                discovery::Event::Joined(room_name) => {
                    pending.push(p2p::NetCommand::Subscribe {
                        topic: room_sync::sync_topic(game, room_name),
                    });
                    let (prefix, lobby_status) = room_prefix_and_lobby_status(
                        status,
                        game,
                        room_name,
                        self_peer_id.as_ref(),
                        ttl,
                    );
                    inputs.push(room_sync::state_machine::Input::Joined {
                        room: room_name.clone(),
                        prefix,
                        members: members.clone(),
                        lobby_status,
                    });
                }
                discovery::Event::Left(room_name) => {
                    pending.push(p2p::NetCommand::Unsubscribe {
                        topic: room_sync::sync_topic(game, room_name),
                    });
                    *state = room_sync::bevy_systems::SyncState(room_sync::state_machine::State {
                        slice_anti_entropy: room_sync::state_machine::SliceAntiEntropyState {
                            max_fetches_per_peer: state.slice_anti_entropy.max_fetches_per_peer,
                            ..Default::default()
                        },
                        kademlia_history: room_sync::state_machine::KademliaHistoryState {
                            is_enabled: state.kademlia_history.is_enabled,
                            max_fetch_retries: state.kademlia_history.max_fetch_retries,
                            per_room_bytes: state.kademlia_history.per_room_bytes,
                            record_ttl: state.kademlia_history.record_ttl,
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                }
                discovery::Event::MembersChanged | discovery::Event::LobbyChanged => {
                    if let Some(room_name) = state.room.clone() {
                        let (prefix, lobby_status) = room_prefix_and_lobby_status(
                            status,
                            game,
                            &room_name,
                            self_peer_id.as_ref(),
                            ttl,
                        );
                        inputs.push(room_sync::state_machine::Input::Members {
                            members: members.clone(),
                            prefix,
                            lobby_status,
                        });
                    }
                }
                _ => {}
            }
        }
    }
    for event in net_events.drain() {
        if let Some(input) = net_event_to_input(&event, &state, &mut clock, game, &*room) {
            inputs.push(input);
        }
    }
    let per_room_bytes = state.kademlia_history.per_room_bytes;
    let record_ttl = state.kademlia_history.record_ttl;
    for input in inputs {
        for output in room_sync::state_machine::update(&mut state, input) {
            perform(
                output,
                &mut *room,
                *clock,
                per_room_bytes,
                record_ttl,
                &mut pending,
                command_sender,
                &mut join_error,
                game,
            );
        }
    }
}

// needed helper: derives the Kademlia history key prefix and lobby liveness for a room from discovery Status
fn room_prefix_and_lobby_status(
    status: Option<&discovery::Status>,
    game: &identity::GameId,
    room_name: &identity::RoomName,
    self_peer_id: Option<&identity::PeerId>,
    ttl: std::time::Duration,
) -> (
    p2p::HistoryKeyPrefix,
    room_sync::state_machine::LobbyRoomStatus,
) {
    let created_at = status
        .and_then(|status| {
            status
                .lobby
                .get(room_name)
                .map(|record| record.created_at)
                .or_else(|| {
                    status
                        .room
                        .as_ref()
                        .filter(|joined| &joined.name == room_name)
                        .map(|joined| joined.created_at)
                })
        })
        .unwrap_or_else(|| discovery::UnixTime::from_secs(0));
    let prefix = p2p::history_key_prefix(game, room_name, created_at);
    let lobby_status = status.map_or(
        room_sync::state_machine::LobbyRoomStatus {
            is_room_alive_in_lobby: true,
            has_other_lobby_holders: false,
        },
        |status| {
            let Some(record) = status.lobby.get(room_name) else {
                let is_the_contract_still_publishing_the_room =
                    status.room.as_ref().is_some_and(|joined| {
                        joined.name == *room_name && joined.phase == discovery::RoomPhase::Joining
                    });
                return room_sync::state_machine::LobbyRoomStatus {
                    is_room_alive_in_lobby: is_the_contract_still_publishing_the_room,
                    has_other_lobby_holders: false,
                };
            };
            let now = record
                .members
                .values()
                .map(|heartbeat| heartbeat.updated_at)
                .max()
                .unwrap_or_else(|| discovery::UnixTime::from_secs(0));
            let is_fresh =
                |heartbeat: &discovery::Heartbeat| now.elapsed_since(heartbeat.updated_at) <= ttl;
            let live_members: Vec<(&identity::PeerId, &discovery::Heartbeat)> = record
                .members
                .iter()
                .filter(|(_, heartbeat)| {
                    heartbeat.presence != discovery::Presence::Quit && is_fresh(heartbeat)
                })
                .collect();
            let live_count = live_members.len();
            let other_live_count = live_members
                .iter()
                .filter(|(peer, _)| Some(*peer) != self_peer_id)
                .count();
            let has_other_history_holder = live_members.iter().any(|(peer, heartbeat)| {
                Some(*peer) != self_peer_id
                    && heartbeat.presence == discovery::Presence::HoldingHistory
            });
            room_sync::state_machine::LobbyRoomStatus {
                is_room_alive_in_lobby: live_count > 0,
                has_other_lobby_holders: other_live_count > 0 || has_other_history_holder,
            }
        },
    );
    (prefix, lobby_status)
}

// needed helper: one net event as a reducer input, if it concerns this room
fn net_event_to_input<Replica: room_sync::RoomState>(
    event: &p2p::NetEvent,
    state: &room_sync::bevy_systems::SyncState<Replica::Op>,
    clock: &mut room_sync::LamportClock,
    game: &identity::GameId,
    replica: &Replica,
) -> Option<room_sync::state_machine::Input<Replica::Op>> {
    match event {
        p2p::NetEvent::Gossip { topic, from, data } => {
            let expected = state
                .room
                .as_ref()
                .map(|room| room_sync::sync_topic(game, room));
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

// needed helper: executes one reducer output
fn perform<Replica: room_sync::RoomState>(
    output: room_sync::state_machine::Output<Replica::Op>,
    room: &mut Replica,
    clock: room_sync::LamportClock,
    per_room_bytes: p2p::HistoryByteCount,
    record_ttl: std::time::Duration,
    pending: &mut p2p::PendingNetCommands,
    command_sender: Option<&discovery::CommandSender>,
    join_error: &mut Option<ResMut<discovery::ui::JoinError>>,
    game: &identity::GameId,
) {
    use p2p::NetCommand;
    use room_sync::state_machine::Output;
    match output {
        Output::FetchSnapshotPartFromRoomMember { peer, room, part } => {
            pending.push(NetCommand::FetchSnapshot {
                peer_id: peer,
                room,
                part,
            });
        }
        Output::SendSnapshotPartToRoomMember {
            id,
            room: name,
            part,
        } => {
            pending.push(NetCommand::RespondSnapshot {
                id,
                frame: snapshot_frame(room, name, part),
            });
        }
        Output::Merge { bytes } => {
            if let Ok(snapshot) = bincode::deserialize::<Replica::Snapshot>(&bytes) {
                room.merge(snapshot);
            }
        }
        Output::Apply { meta, op } => room.apply(meta, &op),
        Output::GetHistoryRecord { id } => pending.push(NetCommand::GetHistoryRecord { id }),
        Output::MergeAndWriteHistory {
            prefix,
            stored_manifest,
            stored_snapshot,
        } => {
            pending.extend(room_sync::merge_and_write_history_records(
                room,
                &prefix,
                stored_manifest.as_ref(),
                stored_snapshot.as_deref(),
                u64::from(*clock),
                per_room_bytes,
                record_ttl,
            ));
        }
        Output::RoomGone { room } => {
            if let Some(err) = join_error {
                ***err = Some(format!(
                    "Room {} no longer exists (all players left)",
                    room.as_str()
                ));
            }
            if let Some(sender) = command_sender {
                sender.send(discovery::Command::Leave).ok();
            }
        }
        Output::HistoryUnavailable { room: _ } => {
            if let Some(err) = join_error {
                ***err = Some(String::from("Couldn't get room history"));
            }
            if let Some(sender) = command_sender {
                sender.send(discovery::Command::Leave).ok();
            }
        }
        Output::PublishSliceContentHashes { room, hashes } => {
            let msg =
                room_sync::ReplicaSyncTopicMessage::<Replica::Op>::CompareSliceContentHashes {
                    room: room.clone(),
                    hashes,
                };
            if let Ok(data) = bincode::serialize(&msg) {
                pending.push(NetCommand::Publish {
                    topic: room_sync::sync_topic(game, &room),
                    data,
                });
            }
        }
        Output::FetchReplicaSlice { peer, room, index } => {
            pending.push(NetCommand::FetchReplicaSlice {
                peer_id: peer,
                room,
                index,
            });
        }
        Output::SendReplicaSlice {
            id,
            room: name,
            index,
        } => {
            let frame = bincode::serialize(&room.slice_snapshot(index)).map_or_else(
                |_| p2p::SnapshotResponse::Missing { room: name.clone() },
                |data| p2p::SnapshotResponse::ReplicaSlice {
                    room: name.clone(),
                    index,
                    data,
                },
            );
            pending.push(NetCommand::RespondSnapshot { id, frame });
        }
    }
}

// needed helper: one snapshot part, or Missing when the index is out of range
fn snapshot_frame<Replica: room_sync::RoomState>(
    room: &Replica,
    room_name: identity::RoomName,
    part: u32,
) -> p2p::SnapshotResponse {
    let Ok(bytes) = bincode::serialize(&room.snapshot()) else {
        return p2p::SnapshotResponse::Missing { room: room_name };
    };
    let parts = room_sync::split_parts(&bytes, room_sync::PART_BYTES);
    let total = u32::try_from(parts.len()).unwrap_or(u32::MAX);
    let data = usize::try_from(part)
        .ok()
        .and_then(|index| parts.get(index).cloned());
    match data {
        Some(data) => p2p::SnapshotResponse::Part {
            room: room_name,
            part,
            total,
            data,
        },
        None => p2p::SnapshotResponse::Missing { room: room_name },
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::Duration;

    use bevy::prelude::*;
    use derive_more::{Deref, DerefMut};
    use serde::{Deserialize, Serialize};

    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::plugin;
    use crate::room_sync;

    const SETTLE: Duration = Duration::from_millis(20);
    const MAX_ROUNDS: usize = 400;

    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    struct Op {
        key: u8,
        value: u32,
    }

    #[derive(Resource, Default, Debug, Clone, PartialEq, Eq, Deref, DerefMut)]
    struct Map {
        entries: BTreeMap<u8, u32>,
    }

    impl room_sync::RoomState for Map {
        type Op = Op;
        type Snapshot = Vec<(u8, u32)>;

        fn apply(&mut self, _meta: room_sync::OpMeta, op: &Self::Op) {
            self.entries.insert(op.key, op.value);
        }

        fn snapshot(&self) -> Self::Snapshot {
            self.entries
                .iter()
                .map(|(key, value)| (*key, *value))
                .collect()
        }

        fn merge(&mut self, snapshot: Self::Snapshot) {
            for (key, value) in snapshot {
                self.entries.insert(key, value);
            }
        }

        fn slice_content_hashes(&self) -> room_sync::ReplicaSliceContentHashes {
            let mut by_slice: BTreeMap<room_sync::ReplicaSliceIndex, Vec<(u8, u32)>> =
                BTreeMap::new();
            for (key, value) in &self.entries {
                by_slice
                    .entry(slice_of(*key))
                    .or_default()
                    .push((*key, *value));
            }
            by_slice
                .into_iter()
                .map(|(index, entries)| {
                    let bytes = bincode::serialize(&entries).unwrap_or_default();
                    (
                        index,
                        room_sync::ReplicaSliceContentHash::from_bytes(&bytes),
                    )
                })
                .collect()
        }

        fn slice_snapshot(&self, index: room_sync::ReplicaSliceIndex) -> Self::Snapshot {
            self.entries
                .iter()
                .filter(|(key, _)| slice_of(**key) == index)
                .map(|(key, value)| (*key, *value))
                .collect()
        }
    }

    fn slice_of(key: u8) -> room_sync::ReplicaSliceIndex {
        room_sync::ReplicaSliceIndex(u32::from(key / 16))
    }

    fn node(game: &identity::GameId) -> App {
        node_with(game, Some(room_sync::AntiEntropyConfig::default()), None)
    }

    fn node_with(
        game: &identity::GameId,
        anti_entropy: Option<room_sync::AntiEntropyConfig>,
        history: Option<room_sync::HistoryConfig>,
    ) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin::P2PPlugin(
            plugin::Config::<u8>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled)
                .with_game(game.clone()),
        ));
        app.add_plugins(
            room_sync::SyncPlugin::<Map>::new(game.clone())
                .with_anti_entropy_config(anti_entropy)
                .with_history_config(history),
        );
        app.add_message::<discovery::Event>();
        app.add_message::<discovery::Command>();
        app.init_resource::<discovery::Status>();
        app.init_resource::<discovery::ui::JoinError>();
        app.update();
        app
    }

    fn dialable(peer: &identity::Peer) -> identity::Peer {
        identity::Peer {
            id: peer.id.clone(),
            addrs: peer
                .addrs
                .iter()
                .map(|addr| identity::PeerAddr(addr.replace("/ip4/0.0.0.0/", "/ip4/127.0.0.1/")))
                .collect(),
        }
    }

    fn ready(app: &App) -> Option<identity::Peer> {
        app.world()
            .resource::<p2p::ReadyChannel>()
            .ready_rx
            .borrow()
            .clone()
    }

    fn status_with_created_at(
        room: &identity::RoomName,
        created_at: discovery::UnixTime,
        connected_members: Vec<identity::PeerId>,
        lobby_members: BTreeMap<identity::PeerId, discovery::Presence>,
    ) -> discovery::Status {
        let mut lobby = discovery::Lobby::default();
        if !lobby_members.is_empty() {
            lobby.insert(
                room.clone(),
                discovery::Room {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at,
                    members: lobby_members
                        .into_iter()
                        .map(|(peer, presence)| {
                            (
                                peer,
                                discovery::Heartbeat {
                                    addrs: Vec::new(),
                                    updated_at: created_at,
                                    display_name: None,
                                    presence,
                                },
                            )
                        })
                        .collect(),
                },
            );
        }
        discovery::Status {
            lobby,
            room: Some(discovery::JoinedRoom {
                name: room.clone(),
                created_at,
                phase: discovery::RoomPhase::Ready,
                members: connected_members
                    .into_iter()
                    .map(|peer| {
                        (
                            peer,
                            discovery::Member {
                                heartbeat: discovery::Heartbeat {
                                    addrs: Vec::new(),
                                    updated_at: created_at,
                                    display_name: None,
                                    presence: discovery::Presence::InRoom,
                                },
                                status: discovery::LinkStatus::Connected,
                            },
                        )
                    })
                    .collect(),
            }),
            freenet_peers: Vec::new(),
            is_lobby_attached: true,
        }
    }

    fn status(room: &identity::RoomName, members: Vec<identity::PeerId>) -> discovery::Status {
        let lobby_members = members
            .iter()
            .map(|peer| (peer.clone(), discovery::Presence::InRoom))
            .collect();
        status_with_created_at(
            room,
            discovery::UnixTime::from_secs(1),
            members,
            lobby_members,
        )
    }

    fn join(app: &mut App, room: &identity::RoomName, members: Vec<identity::PeerId>) {
        app.insert_resource(status(room, members));
        app.world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(discovery::Event::Joined(room.clone()));
    }

    fn wait_ready(app: &mut App) -> identity::Peer {
        for _ in 0..MAX_ROUNDS {
            app.update();
            if let Some(peer) = ready(app) {
                return peer;
            }
            std::thread::sleep(SETTLE);
        }
        panic!("node never became ready");
    }

    fn wait_connected(
        app: &mut App,
        events: &mut tokio::sync::broadcast::Receiver<p2p::NetEvent>,
        peer: &identity::PeerId,
    ) {
        for _ in 0..MAX_ROUNDS {
            app.update();
            while let Ok(event) = events.try_recv() {
                if let p2p::NetEvent::PeerConnected(id) = event
                    && &id == peer
                {
                    return;
                }
            }
            std::thread::sleep(SETTLE);
        }
        panic!("peer never connected");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_usage() {
        let mut app = node(&identity::GameId::from("game"));
        app.update();
        assert!(
            app.world()
                .get_resource::<room_sync::bevy_systems::SyncState<Op>>()
                .is_some()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn late_joiner_converges_over_the_new_behaviour() {
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-a");
        let mut incumbent = node(&game);
        let mut joiner = node(&game);

        let a = wait_ready(&mut incumbent);
        let _b = wait_ready(&mut joiner);

        let mut joiner_events = joiner
            .world()
            .resource::<p2p::NetEventChannel>()
            .subscribe();
        joiner
            .world()
            .resource::<p2p::NetCommandSender>()
            .send(p2p::NetCommand::Dial(dialable(&a)))
            .expect("dial");
        wait_connected(&mut joiner, &mut joiner_events, &a.id);

        incumbent
            .world_mut()
            .resource_mut::<Map>()
            .entries
            .insert(7, 42);
        join(&mut incumbent, &room, Vec::new());
        join(&mut joiner, &room, vec![a.id.clone()]);

        let mut saw_part = false;
        for _ in 0..MAX_ROUNDS {
            incumbent.update();
            joiner.update();
            while let Ok(event) = joiner_events.try_recv() {
                if matches!(event, p2p::NetEvent::SnapshotPart { .. }) {
                    saw_part = true;
                }
            }
            let expected = incumbent.world().resource::<Map>().clone();
            let actual = joiner.world().resource::<Map>().clone();
            if actual == expected && !expected.entries.is_empty() {
                assert!(saw_part, "converged without a snapshot part");
                assert_eq!(actual.entries.get(&7), Some(&42));
                return;
            }
            tokio::time::sleep(SETTLE).await;
        }
        panic!(
            "joiner did not converge: {:?}",
            joiner.world().resource::<Map>()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn transport_only_three_apps_converge_after_dropping_30_percent_of_ops() {
        let interval = Duration::from_millis(500);
        let game = identity::GameId::from("game");
        let room = identity::RoomName::from("room-ae");
        let config = Some(room_sync::AntiEntropyConfig {
            interval,
            max_slice_fetches_per_peer: 8,
        });
        let mut apps = vec![
            node_with(&game, config, None),
            node_with(&game, config, None),
            node_with(&game, config, None),
        ];
        let peers: Vec<identity::Peer> = apps.iter_mut().map(wait_ready).collect();
        for (dialer, target) in [(1usize, 0usize), (2, 0), (2, 1)] {
            let mut events = apps[dialer]
                .world()
                .resource::<p2p::NetEventChannel>()
                .subscribe();
            apps[dialer]
                .world()
                .resource::<p2p::NetCommandSender>()
                .send(p2p::NetCommand::Dial(dialable(&peers[target])))
                .expect("dial");
            wait_connected(&mut apps[dialer], &mut events, &peers[target].id);
        }
        for (index, app) in apps.iter_mut().enumerate() {
            let others = peers
                .iter()
                .enumerate()
                .filter(|(other, _)| *other != index)
                .map(|(_, peer)| peer.id.clone())
                .collect();
            join(app, &room, others);
        }
        for _ in 0..50 {
            for app in &mut apps {
                app.update();
            }
            tokio::time::sleep(SETTLE).await;
        }

        for key in 0u8..120 {
            let author = usize::from(key) % apps.len();
            let op = Op {
                key,
                value: u32::from(key) * 10,
            };
            if key % 10 < 3 {
                apps[author]
                    .world_mut()
                    .resource_mut::<Map>()
                    .entries
                    .insert(op.key, op.value);
            } else {
                apps[author]
                    .world_mut()
                    .resource_mut::<room_sync::PendingOps<Map>>()
                    .ops
                    .push(op);
            }
            apps[author].update();
        }

        let traffic_stopped = std::time::Instant::now();
        let deadline = interval * 3;
        loop {
            for app in &mut apps {
                app.update();
            }
            let hashes: Vec<room_sync::ReplicaSliceContentHashes> = apps
                .iter()
                .map(|app| {
                    room_sync::RoomState::slice_content_hashes(app.world().resource::<Map>())
                })
                .collect();
            let full = apps
                .iter()
                .all(|app| app.world().resource::<Map>().entries.len() == 120);
            if full && hashes.windows(2).all(|pair| pair.first() == pair.last()) {
                return;
            }
            assert!(
                traffic_stopped.elapsed() <= deadline,
                "not converged within 3 intervals: sizes {:?}",
                apps.iter()
                    .map(|app| app.world().resource::<Map>().entries.len())
                    .collect::<Vec<_>>()
            );
            tokio::time::sleep(SETTLE).await;
        }
    }

    fn connect_and_seed_kad(app: &mut App, peer: &identity::Peer) {
        let mut events = app.world().resource::<p2p::NetEventChannel>().subscribe();
        let sender = app.world().resource::<p2p::NetCommandSender>();
        sender
            .send(p2p::NetCommand::AddKadPeer(peer.clone()))
            .expect("kad");
        sender
            .send(p2p::NetCommand::Dial(peer.clone()))
            .expect("dial");
        wait_connected(app, &mut events, &peer.id);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn holders_keep_history_after_leaving_and_new_joiner_restores_board() {
        let game = identity::GameId::from("game-h3");
        let room = identity::RoomName::from("room-h3");
        let history = Some(room_sync::HistoryConfig {
            write_interval: Duration::from_millis(80),
            record_ttl: Duration::from_secs(90),
            per_room_bytes: p2p::HistoryByteCount::from(64u64 * 1024 * 1024),
            max_fetch_retries: 2,
        });
        let mut peer_a = node_with(&game, None, history);
        let mut peer_b = node_with(&game, None, history);
        let mut peer_c = node_with(&game, None, history);

        let a = dialable(&wait_ready(&mut peer_a));
        let b = dialable(&wait_ready(&mut peer_b));
        let _c = wait_ready(&mut peer_c);
        let mut b_events = peer_b
            .world()
            .resource::<p2p::NetEventChannel>()
            .subscribe();
        connect_and_seed_kad(&mut peer_b, &a);

        let gen1 = discovery::UnixTime::from_secs(10);
        let lobby_ab = BTreeMap::from([
            (a.id.clone(), discovery::Presence::InRoom),
            (b.id.clone(), discovery::Presence::InRoom),
        ]);
        peer_a.insert_resource(status_with_created_at(
            &room,
            gen1,
            Vec::new(),
            BTreeMap::from([(a.id.clone(), discovery::Presence::InRoom)]),
        ));
        peer_a
            .world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(discovery::Event::Joined(room.clone()));
        peer_a
            .world_mut()
            .resource_mut::<Map>()
            .entries
            .insert(9, 99);

        peer_b.insert_resource(status_with_created_at(
            &room,
            gen1,
            vec![a.id.clone()],
            lobby_ab,
        ));
        peer_b
            .world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(discovery::Event::Joined(room.clone()));

        let mut stored = false;
        for _ in 0..MAX_ROUNDS {
            peer_a.update();
            peer_b.update();
            while let Ok(event) = b_events.try_recv() {
                if matches!(event, p2p::NetEvent::HistoryRecordStored { .. }) {
                    stored = true;
                }
            }
            if stored && peer_b.world().resource::<Map>().entries.get(&9) == Some(&99) {
                break;
            }
            tokio::time::sleep(SETTLE).await;
        }
        assert_eq!(peer_b.world().resource::<Map>().entries.get(&9), Some(&99));

        for app in [&mut peer_a, &mut peer_b] {
            app.world_mut()
                .resource_mut::<Messages<discovery::Event>>()
                .write(discovery::Event::Left(room.clone()));
            app.update();
        }

        connect_and_seed_kad(&mut peer_c, &a);
        connect_and_seed_kad(&mut peer_c, &b);

        let holders = BTreeMap::from([
            (a.id.clone(), discovery::Presence::HoldingHistory),
            (b.id.clone(), discovery::Presence::HoldingHistory),
        ]);
        peer_c.insert_resource(status_with_created_at(&room, gen1, Vec::new(), holders));
        peer_c
            .world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(discovery::Event::Joined(room.clone()));

        for _ in 0..MAX_ROUNDS {
            peer_a.update();
            peer_b.update();
            peer_c.update();
            if peer_c.world().resource::<Map>().entries.get(&9) == Some(&99) {
                break;
            }
            tokio::time::sleep(SETTLE).await;
        }
        assert_eq!(peer_c.world().resource::<Map>().entries.get(&9), Some(&99));
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn recreated_room_generation_starts_empty() {
        let game = identity::GameId::from("game-recreate");
        let room = identity::RoomName::from("room-recreate");
        let history = Some(room_sync::HistoryConfig::default());
        let mut peer_d = node_with(&game, None, history);
        let d = wait_ready(&mut peer_d);
        let gen2 = discovery::UnixTime::from_secs(20);
        peer_d.insert_resource(status_with_created_at(
            &room,
            gen2,
            Vec::new(),
            BTreeMap::from([(d.id, discovery::Presence::InRoom)]),
        ));
        peer_d
            .world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(discovery::Event::Joined(room));
        for _ in 0..30 {
            peer_d.update();
            if peer_d
                .world()
                .resource::<room_sync::bevy_systems::SyncState<Op>>()
                .phase
                == room_sync::state_machine::Phase::Ready
            {
                break;
            }
            tokio::time::sleep(SETTLE).await;
        }
        assert_eq!(
            peer_d
                .world()
                .resource::<room_sync::bevy_systems::SyncState<Op>>()
                .phase,
            room_sync::state_machine::Phase::Ready
        );
        assert!(peer_d.world().resource::<Map>().entries.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_room_the_contract_has_not_confirmed_yet_is_not_gone() {
        let game = identity::GameId::from("game-new");
        let room = identity::RoomName::from("room-new");
        let history = Some(room_sync::HistoryConfig::default());
        let mut creator = node_with(&game, None, history);
        wait_ready(&mut creator);
        let mut status = status_with_created_at(
            &room,
            discovery::UnixTime::from_secs(1),
            Vec::new(),
            BTreeMap::new(),
        );
        if let Some(joined) = status.room.as_mut() {
            joined.phase = discovery::RoomPhase::Joining;
        }
        creator.insert_resource(status);
        creator
            .world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(discovery::Event::Joined(room.clone()));
        for _ in 0..30 {
            creator.update();
            tokio::time::sleep(SETTLE).await;
        }
        assert!(
            !log_contains_joined_error(&creator),
            "a room the contract has not confirmed yet is not gone"
        );
    }

    fn log_contains_joined_error(app: &App) -> bool {
        app.world().resource::<discovery::ui::JoinError>().is_some()
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn dead_room_and_mid_fetch_quit_emit_room_gone() {
        let game = identity::GameId::from("game-gone");
        let room = identity::RoomName::from("room-gone");
        let history = Some(room_sync::HistoryConfig {
            write_interval: Duration::from_secs(30),
            record_ttl: Duration::from_secs(90),
            per_room_bytes: p2p::HistoryByteCount::from(64u64 * 1024 * 1024),
            max_fetch_retries: 3,
        });
        let mut joiner = node_with(&game, None, history);
        wait_ready(&mut joiner);
        joiner.insert_resource(status_with_created_at(
            &room,
            discovery::UnixTime::from_secs(1),
            Vec::new(),
            BTreeMap::new(),
        ));
        joiner
            .world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(discovery::Event::Joined(room.clone()));

        let mut saw_room_gone = false;
        for _ in 0..50 {
            joiner.update();
            if joiner
                .world()
                .resource::<discovery::ui::JoinError>()
                .as_deref()
                == Some("Room room-gone no longer exists (all players left)")
            {
                saw_room_gone = true;
                break;
            }
            tokio::time::sleep(SETTLE).await;
        }
        assert!(saw_room_gone);

        let mut mid_joiner = node_with(&game, None, history);
        wait_ready(&mut mid_joiner);
        let holder_id = identity::PeerId::from("holder");
        mid_joiner.insert_resource(status_with_created_at(
            &room,
            discovery::UnixTime::from_secs(2),
            Vec::new(),
            BTreeMap::from([(holder_id, discovery::Presence::HoldingHistory)]),
        ));
        mid_joiner
            .world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(discovery::Event::Joined(room.clone()));
        mid_joiner.update();
        assert_eq!(
            mid_joiner
                .world()
                .resource::<room_sync::bevy_systems::SyncState<Op>>()
                .phase,
            room_sync::state_machine::Phase::FetchingHistoryFromKademlia
        );

        mid_joiner.insert_resource(status_with_created_at(
            &room,
            discovery::UnixTime::from_secs(2),
            Vec::new(),
            BTreeMap::new(),
        ));
        mid_joiner
            .world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(discovery::Event::LobbyChanged);
        mid_joiner.update();
        assert_eq!(
            mid_joiner
                .world()
                .resource::<discovery::ui::JoinError>()
                .as_deref(),
            Some("Room room-gone no longer exists (all players left)")
        );
    }
}
