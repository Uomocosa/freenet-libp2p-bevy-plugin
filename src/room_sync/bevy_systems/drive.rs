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
        p2p::NetEvent::SnapshotFailed { .. } => Some(room_sync::state_machine::Input::Failed),
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
    }

    fn node(game: &identity::GameId) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin::P2PPlugin(
            plugin::Config::<u8>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled)
                .with_game(game.clone()),
        ));
        app.add_plugins(room_sync::SyncPlugin::<Map>::new(game.clone()));
        app.add_message::<discovery::Event>();
        app.init_resource::<discovery::Snapshot>();
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

    fn snapshot(room: &identity::RoomName, members: Vec<identity::PeerId>) -> discovery::Snapshot {
        discovery::Snapshot {
            lobby: discovery::Lobby::default(),
            room: Some(discovery::JoinedRoom {
                name: room.clone(),
                created_at: discovery::UnixTime::from_secs(1),
                phase: discovery::RoomPhase::Ready,
                members: members
                    .into_iter()
                    .map(|peer| {
                        (
                            peer,
                            discovery::Member {
                                heartbeat: discovery::Heartbeat {
                                    addrs: Vec::new(),
                                    updated_at: discovery::UnixTime::from_secs(1),
                                    display_name: None,
                                },
                                status: discovery::LinkStatus::Connected,
                            },
                        )
                    })
                    .collect(),
            }),
            freenet_peers: Vec::new(),
            lobby_attached: true,
        }
    }

    fn join(app: &mut App, room: &identity::RoomName, members: Vec<identity::PeerId>) {
        app.insert_resource(snapshot(room, members));
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
}
