use bevy::prelude::*;

use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::room_sync;

pub fn drive<Replica: room_sync::RoomState>(
    room_events: Option<MessageReader<discovery::Event>>,
    command_sender: Option<Res<discovery::CommandSender>>,
    mut join_error: Option<ResMut<discovery::ui::JoinError>>,
    discovery_status: Option<Res<discovery::Status>>,
    net_events: Res<room_sync::bevy_systems::NetEvents>,
    game: Res<room_sync::SyncGame>,
    mut clock: ResMut<room_sync::LamportClock>,
    mut state: ResMut<room_sync::bevy_systems::SyncState<Replica::Op>>,
    mut room: ResMut<Replica>,
    mut pending: ResMut<p2p::PendingNetCommands>,
) {
    let discovery_status = discovery_status.map(Res::into_inner);
    let command_sender = command_sender.map(Res::into_inner);
    let net_events = net_events.into_inner();
    let game = game.into_inner();
    let members: Vec<identity::PeerId> = discovery_status.map_or_else(Vec::new, |status| {
        status.connected_members().cloned().collect()
    });
    let mut inputs: Vec<room_sync::state_machine::Input<Replica::Op>> = Vec::new();
    if let Some(mut room_events) = room_events {
        for event in room_events.read() {
            match event {
                discovery::Event::Joined(room_name) => {
                    pending.push(p2p::NetCommand::Subscribe {
                        topic: identity::Topic::from_room_sync(game, room_name),
                    });
                    let (prefix, lobby_status) = discovery::Status::room_prefix_and_lobby_status(
                        discovery_status,
                        game,
                        room_name,
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
                        topic: identity::Topic::from_room_sync(game, room_name),
                    });
                    *state = room_sync::bevy_systems::SyncState(
                        room_sync::state_machine::ReplicaState {
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
                        },
                    );
                }
                discovery::Event::MembersChanged | discovery::Event::LobbyChanged => {
                    if let Some(room_name) = state.room.clone() {
                        let (prefix, lobby_status) =
                            discovery::Status::room_prefix_and_lobby_status(
                                discovery_status,
                                game,
                                &room_name,
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
        if let Some(input) = room_sync::bevy_systems::SyncState::<Replica::Op>::net_event_to_input(
            &event, &state, &mut clock, game, &*room,
        ) {
            inputs.push(input);
        }
    }
    let per_room_bytes = state.kademlia_history.per_room_bytes;
    let record_ttl = state.kademlia_history.record_ttl;
    for input in inputs {
        for output in state.update(input) {
            room_sync::bevy_systems::SyncState::<Replica::Op>::perform(
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
                        room_sync::ReplicaSliceContentHash::from(bytes.as_slice()),
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
        lobby_members: BTreeMap<identity::PeerId, discovery::RoomMembership>,
    ) -> discovery::Status {
        let mut lobby = discovery::Lobby::default();
        if !lobby_members.is_empty() {
            lobby.insert(
                room.clone(),
                discovery::LobbyRoom {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at,
                    members: lobby_members
                        .into_iter()
                        .map(|(peer, membership)| {
                            (
                                peer.clone(),
                                discovery::RoomMember {
                                    peer: identity::Peer {
                                        id: peer,
                                        addrs: Vec::new(),
                                    },
                                    updated_at: created_at,
                                    display_name: None,
                                    membership,
                                },
                            )
                        })
                        .collect(),
                },
            );
        }
        let connected: std::collections::BTreeSet<identity::PeerId> =
            connected_members.iter().cloned().collect();
        discovery::Status {
            lobby,
            connected,
            room: Some(discovery::JoinedRoom {
                name: room.clone(),
                created_at,
                phase: discovery::JoinPhase::Ready,
                members: connected_members
                    .into_iter()
                    .map(|peer| {
                        (
                            peer.clone(),
                            discovery::RoomMember {
                                peer: identity::Peer {
                                    id: peer,
                                    addrs: Vec::new(),
                                },
                                updated_at: created_at,
                                display_name: None,
                                membership: discovery::RoomMembership::Occupant,
                            },
                        )
                    })
                    .collect(),
            }),
            freenet_peers: Vec::new(),
            is_lobby_attached: true,
            self_peer_id: None,
            lease_ttl: discovery::HolderHeartbeatIntervals::default().ttl,
        }
    }

    fn status(room: &identity::RoomName, members: Vec<identity::PeerId>) -> discovery::Status {
        let lobby_members = members
            .iter()
            .map(|peer| (peer.clone(), discovery::RoomMembership::Occupant))
            .collect();
        status_with_created_at(room, discovery::UnixTime::from(1), members, lobby_members)
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

        let gen1 = discovery::UnixTime::from(10);
        let lobby_ab = BTreeMap::from([
            (a.id.clone(), discovery::RoomMembership::Occupant),
            (b.id.clone(), discovery::RoomMembership::Occupant),
        ]);
        peer_a.insert_resource(status_with_created_at(
            &room,
            gen1,
            Vec::new(),
            BTreeMap::from([(a.id.clone(), discovery::RoomMembership::Occupant)]),
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
            (a.id.clone(), discovery::RoomMembership::HistoryHolder),
            (b.id.clone(), discovery::RoomMembership::HistoryHolder),
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
        let gen2 = discovery::UnixTime::from(20);
        peer_d.insert_resource(status_with_created_at(
            &room,
            gen2,
            Vec::new(),
            BTreeMap::from([(d.id, discovery::RoomMembership::Occupant)]),
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
            discovery::UnixTime::from(1),
            Vec::new(),
            BTreeMap::new(),
        );
        if let Some(joined) = status.room.as_mut() {
            joined.phase = discovery::JoinPhase::Joining {
                since: discovery::UnixTime::from(0),
            };
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
            discovery::UnixTime::from(1),
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
            discovery::UnixTime::from(2),
            Vec::new(),
            BTreeMap::from([(holder_id, discovery::RoomMembership::HistoryHolder)]),
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
            discovery::UnixTime::from(2),
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
