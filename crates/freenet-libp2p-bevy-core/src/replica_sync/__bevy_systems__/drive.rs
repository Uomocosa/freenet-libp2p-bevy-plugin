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
#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::time::Duration;

    use bevy::prelude::*;
    use derive_more::{Deref, DerefMut};
    use serde::{Deserialize, Serialize};

    use crate::identity;
    use crate::p2p;
    use crate::peers;
    use crate::plugin;
    use crate::replica_sync;
    use crate::time;

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

    impl replica_sync::traits::ReplicatedState for Map {
        type Op = Op;
        type Snapshot = Vec<(u8, u32)>;

        fn apply(&mut self, _meta: replica_sync::OpMeta, op: &Self::Op) {
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

        fn slice_content_hashes(&self) -> replica_sync::ReplicaSliceContentHashes {
            let mut by_slice: BTreeMap<replica_sync::ReplicaSliceIndex, Vec<(u8, u32)>> =
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
                        replica_sync::ReplicaSliceContentHash::from(bytes.as_slice()),
                    )
                })
                .collect()
        }

        fn slice_snapshot(&self, index: replica_sync::ReplicaSliceIndex) -> Self::Snapshot {
            self.entries
                .iter()
                .filter(|(key, _)| slice_of(**key) == index)
                .map(|(key, value)| (*key, *value))
                .collect()
        }
    }

    fn slice_of(key: u8) -> replica_sync::ReplicaSliceIndex {
        replica_sync::ReplicaSliceIndex(u32::from(key / 16))
    }

    fn node(game: &identity::GameId) -> App {
        node_with(game, Some(replica_sync::AntiEntropyConfig::default()), None)
    }

    fn node_with(
        game: &identity::GameId,
        anti_entropy: Option<replica_sync::AntiEntropyConfig>,
        history: Option<replica_sync::HistoryConfig>,
    ) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_plugins(plugin::P2PPlugin(
            plugin::Config::<u8>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled)
                .with_game(game.clone()),
        ));
        app.add_plugins(
            replica_sync::SyncPlugin::<Map>::new(game.clone())
                .with_anti_entropy_config(anti_entropy)
                .with_history_config(history),
        );
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

    fn join_with(
        app: &mut App,
        zone: &identity::ZoneName,
        members: Vec<identity::PeerId>,
        prefix: p2p::HistoryKeyPrefix,
        liveness: replica_sync::state_machine::ZoneLiveness,
    ) {
        let zone_entity = app.world_mut().spawn((zone.clone(), prefix, liveness)).id();
        let me_entities: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<peers::Me>>()
            .iter(app.world())
            .collect();
        if me_entities.is_empty() {
            let self_id = ready(app).map_or_else(|| identity::PeerId::from("me"), |peer| peer.id);
            app.world_mut().spawn((
                peers::Peer::from(self_id),
                peers::Me,
                peers::InZone(zone_entity),
            ));
        } else {
            for entity in me_entities {
                app.world_mut()
                    .entity_mut(entity)
                    .insert(peers::InZone(zone_entity));
            }
        }
        for member in members {
            app.world_mut().spawn((
                peers::Peer::from(member),
                peers::InZone(zone_entity),
                peers::Connected,
            ));
        }
    }

    fn join(
        app: &mut App,
        game: &identity::GameId,
        zone: &identity::ZoneName,
        members: Vec<identity::PeerId>,
    ) {
        let prefix = p2p::HistoryKeyPrefix::new(game, zone, time::UnixTime::now());
        let listed = !members.is_empty();
        join_with(
            app,
            zone,
            members,
            prefix,
            replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed: listed,
                has_other_history_holders: listed,
            },
        );
    }

    fn leave(app: &mut App) {
        let me_entities: Vec<Entity> = app
            .world_mut()
            .query_filtered::<Entity, With<peers::Me>>()
            .iter(app.world())
            .collect();
        for entity in me_entities {
            app.world_mut().entity_mut(entity).remove::<peers::InZone>();
        }
    }

    fn set_liveness(app: &mut App, liveness: replica_sync::state_machine::ZoneLiveness) {
        let target: Option<Entity> = app
            .world_mut()
            .query_filtered::<&peers::InZone, With<peers::Me>>()
            .iter(app.world())
            .next()
            .map(|in_zone| **in_zone);
        if let Some(entity) = target {
            app.world_mut().entity_mut(entity).insert(liveness);
        }
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
        events: &mut tokio::sync::broadcast::Receiver<p2p::Libp2pEvent>,
        peer: &identity::PeerId,
    ) {
        for _ in 0..MAX_ROUNDS {
            app.update();
            while let Ok(event) = events.try_recv() {
                if let p2p::Libp2pEvent::PeerConnected(id) = event
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
                .get_resource::<replica_sync::bevy_systems::SyncState<Op>>()
                .is_some()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn late_joiner_converges_over_the_new_behaviour() {
        let game = identity::GameId::from("game");
        let zone = identity::ZoneName::from("zone-a");
        let mut incumbent = node(&game);
        let mut joiner = node(&game);

        let a = wait_ready(&mut incumbent);
        let _b = wait_ready(&mut joiner);

        let mut joiner_events = joiner
            .world()
            .resource::<p2p::Libp2pEventChannel>()
            .subscribe();
        joiner
            .world()
            .resource::<p2p::Libp2pCommandSender>()
            .send(p2p::Libp2pCommand::Dial(dialable(&a)))
            .expect("dial");
        wait_connected(&mut joiner, &mut joiner_events, &a.id);

        incumbent
            .world_mut()
            .resource_mut::<Map>()
            .entries
            .insert(7, 42);
        join(&mut incumbent, &game, &zone, Vec::new());
        join(&mut joiner, &game, &zone, vec![a.id.clone()]);

        let mut saw_part = false;
        for _ in 0..MAX_ROUNDS {
            incumbent.update();
            joiner.update();
            while let Ok(event) = joiner_events.try_recv() {
                if matches!(event, p2p::Libp2pEvent::SnapshotPart { .. }) {
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
        let zone = identity::ZoneName::from("zone-ae");
        let config = Some(replica_sync::AntiEntropyConfig {
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
                .resource::<p2p::Libp2pEventChannel>()
                .subscribe();
            apps[dialer]
                .world()
                .resource::<p2p::Libp2pCommandSender>()
                .send(p2p::Libp2pCommand::Dial(dialable(&peers[target])))
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
            join(app, &game, &zone, others);
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
                    .resource_mut::<replica_sync::PendingOps<Map>>()
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
            let hashes: Vec<replica_sync::ReplicaSliceContentHashes> = apps
                .iter()
                .map(|app| {
                    replica_sync::traits::ReplicatedState::slice_content_hashes(
                        app.world().resource::<Map>(),
                    )
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
        let mut events = app
            .world()
            .resource::<p2p::Libp2pEventChannel>()
            .subscribe();
        let sender = app.world().resource::<p2p::Libp2pCommandSender>();
        sender
            .send(p2p::Libp2pCommand::AddKadPeer(peer.clone()))
            .expect("kad");
        sender
            .send(p2p::Libp2pCommand::Dial(peer.clone()))
            .expect("dial");
        wait_connected(app, &mut events, &peer.id);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn holders_keep_history_after_leaving_and_new_joiner_restores_board() {
        let game = identity::GameId::from("game-h3");
        let zone = identity::ZoneName::from("zone-h3");
        let history = Some(replica_sync::HistoryConfig {
            write_interval: Duration::from_millis(80),
            record_ttl: Duration::from_secs(90),
            per_zone_bytes: p2p::HistoryByteCount::from(64u64 * 1024 * 1024),
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
            .resource::<p2p::Libp2pEventChannel>()
            .subscribe();
        connect_and_seed_kad(&mut peer_b, &a);

        let gen1 = time::UnixTime::now();
        let prefix = p2p::HistoryKeyPrefix::new(&game, &zone, gen1);
        let present = replica_sync::state_machine::ZoneLiveness {
            is_zone_still_listed: true,
            has_other_history_holders: true,
        };
        join_with(&mut peer_a, &zone, Vec::new(), prefix.clone(), present);
        peer_a
            .world_mut()
            .resource_mut::<Map>()
            .entries
            .insert(9, 99);

        join_with(
            &mut peer_b,
            &zone,
            vec![a.id.clone()],
            prefix.clone(),
            present,
        );

        let mut stored = false;
        for _ in 0..MAX_ROUNDS {
            peer_a.update();
            peer_b.update();
            while let Ok(event) = b_events.try_recv() {
                if matches!(event, p2p::Libp2pEvent::HistoryRecordStored { .. }) {
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
            leave(app);
            app.update();
        }

        connect_and_seed_kad(&mut peer_c, &a);
        connect_and_seed_kad(&mut peer_c, &b);

        join_with(&mut peer_c, &zone, Vec::new(), prefix, present);

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
    async fn recreated_zone_generation_starts_empty() {
        let game = identity::GameId::from("game-recreate");
        let zone = identity::ZoneName::from("zone-recreate");
        let history = Some(replica_sync::HistoryConfig::default());
        let mut peer_d = node_with(&game, None, history);
        let _d = wait_ready(&mut peer_d);
        let generation = time::UnixTime::now();
        join_with(
            &mut peer_d,
            &zone,
            Vec::new(),
            p2p::HistoryKeyPrefix::new(&game, &zone, generation),
            replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed: true,
                has_other_history_holders: false,
            },
        );
        for _ in 0..30 {
            peer_d.update();
            if peer_d
                .world()
                .resource::<replica_sync::bevy_systems::SyncState<Op>>()
                .phase
                == replica_sync::state_machine::Phase::Ready
            {
                break;
            }
            tokio::time::sleep(SETTLE).await;
        }
        assert_eq!(
            peer_d
                .world()
                .resource::<replica_sync::bevy_systems::SyncState<Op>>()
                .phase,
            replica_sync::state_machine::Phase::Ready
        );
        assert!(peer_d.world().resource::<Map>().entries.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_zone_the_contract_has_not_confirmed_yet_is_not_gone() {
        let game = identity::GameId::from("game-new");
        let zone = identity::ZoneName::from("zone-new");
        let history = Some(replica_sync::HistoryConfig::default());
        let mut creator = node_with(&game, None, history);
        wait_ready(&mut creator);
        join_with(
            &mut creator,
            &zone,
            Vec::new(),
            p2p::HistoryKeyPrefix::new(&game, &zone, time::UnixTime::from_secs(1)),
            replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed: true,
                has_other_history_holders: false,
            },
        );
        for _ in 0..30 {
            creator.update();
            tokio::time::sleep(SETTLE).await;
        }
        assert!(
            !log_contains_joined_error(&mut creator),
            "a zone the contract has not confirmed yet is not gone"
        );
    }

    fn log_contains_joined_error(app: &mut App) -> bool {
        app.world_mut()
            .resource_mut::<Messages<replica_sync::JoinFailure>>()
            .drain()
            .any(|event| {
                matches!(
                    event,
                    replica_sync::JoinFailure::ZoneGone(_)
                        | replica_sync::JoinFailure::HistoryUnavailable(_)
                )
            })
    }

    fn emitted_zone_gone(app: &mut App, expected: &identity::ZoneName) -> bool {
        app.world_mut()
            .resource_mut::<Messages<replica_sync::JoinFailure>>()
            .drain()
            .any(|event| match event {
                replica_sync::JoinFailure::ZoneGone(zone) => &zone == expected,
                replica_sync::JoinFailure::HistoryUnavailable(_) => false,
            })
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn expired_holder_alone_is_gone_not_empty() {
        let game = identity::GameId::from("game-s4c");
        let zone = identity::ZoneName::from("zone-s4c");
        let history = Some(replica_sync::HistoryConfig {
            write_interval: Duration::from_secs(30),
            record_ttl: Duration::from_secs(90),
            per_zone_bytes: p2p::HistoryByteCount::from(64u64 * 1024 * 1024),
            max_fetch_retries: 3,
        });
        let mut joiner = node_with(&game, None, history);
        let _joiner_peer = wait_ready(&mut joiner);
        join_with(
            &mut joiner,
            &zone,
            Vec::new(),
            p2p::HistoryKeyPrefix::new(&game, &zone, time::UnixTime::EPOCH),
            replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed: false,
                has_other_history_holders: false,
            },
        );

        let mut saw_zone_gone = false;
        for _ in 0..MAX_ROUNDS {
            joiner.update();
            if emitted_zone_gone(&mut joiner, &zone) {
                saw_zone_gone = true;
                break;
            }
            if joiner
                .world()
                .resource::<replica_sync::bevy_systems::SyncState<Op>>()
                .phase
                == replica_sync::state_machine::Phase::Ready
                && joiner.world().resource::<Map>().entries.is_empty()
            {
                panic!("S4c joiner accepted an empty board instead of reporting the zone gone");
            }
            tokio::time::sleep(SETTLE).await;
        }
        assert!(
            saw_zone_gone,
            "expired holder alone must report the zone gone, not an empty board"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn joiner_whose_global_holds_only_itself_sees_the_zone_as_gone() {
        let game = identity::GameId::from("game-s4j");
        let zone = identity::ZoneName::from("zone-s4j");
        let history = Some(replica_sync::HistoryConfig {
            write_interval: Duration::from_secs(30),
            record_ttl: Duration::from_secs(90),
            per_zone_bytes: p2p::HistoryByteCount::from(64u64 * 1024 * 1024),
            max_fetch_retries: 3,
        });
        let mut joiner = node_with(&game, None, history);
        let _joiner_peer = wait_ready(&mut joiner);
        join_with(
            &mut joiner,
            &zone,
            Vec::new(),
            p2p::HistoryKeyPrefix::new(&game, &zone, time::UnixTime::EPOCH),
            replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed: false,
                has_other_history_holders: false,
            },
        );

        let mut saw_zone_gone = false;
        for _ in 0..MAX_ROUNDS {
            joiner.update();
            if emitted_zone_gone(&mut joiner, &zone) {
                saw_zone_gone = true;
                break;
            }
            if joiner
                .world()
                .resource::<replica_sync::bevy_systems::SyncState<Op>>()
                .phase
                == replica_sync::state_machine::Phase::Ready
                && joiner.world().resource::<Map>().entries.is_empty()
            {
                panic!("S4b joiner accepted an empty board instead of reporting the zone gone");
            }
            tokio::time::sleep(SETTLE).await;
        }
        assert!(
            saw_zone_gone,
            "joiner alone in its zone must report the zone gone, not an empty board"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn dead_zone_and_mid_fetch_quit_emit_zone_gone() {
        let game = identity::GameId::from("game-gone");
        let zone = identity::ZoneName::from("zone-gone");
        let history = Some(replica_sync::HistoryConfig {
            write_interval: Duration::from_secs(30),
            record_ttl: Duration::from_secs(90),
            per_zone_bytes: p2p::HistoryByteCount::from(64u64 * 1024 * 1024),
            max_fetch_retries: 3,
        });
        let mut joiner = node_with(&game, None, history);
        wait_ready(&mut joiner);
        join_with(
            &mut joiner,
            &zone,
            Vec::new(),
            p2p::HistoryKeyPrefix::new(&game, &zone, time::UnixTime::from_secs(1)),
            replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed: false,
                has_other_history_holders: false,
            },
        );

        let mut saw_zone_gone = false;
        for _ in 0..50 {
            joiner.update();
            if emitted_zone_gone(&mut joiner, &zone) {
                saw_zone_gone = true;
                break;
            }
            tokio::time::sleep(SETTLE).await;
        }
        assert!(saw_zone_gone);

        let mut mid_joiner = node_with(&game, None, history);
        wait_ready(&mut mid_joiner);
        let generation = time::UnixTime::now();
        join_with(
            &mut mid_joiner,
            &zone,
            Vec::new(),
            p2p::HistoryKeyPrefix::new(&game, &zone, generation),
            replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed: true,
                has_other_history_holders: true,
            },
        );
        mid_joiner.update();
        assert_eq!(
            mid_joiner
                .world()
                .resource::<replica_sync::bevy_systems::SyncState<Op>>()
                .phase,
            replica_sync::state_machine::Phase::FetchingHistoryFromKademlia
        );

        set_liveness(
            &mut mid_joiner,
            replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed: false,
                has_other_history_holders: false,
            },
        );
        mid_joiner.update();
        assert!(
            emitted_zone_gone(&mut mid_joiner, &zone),
            "mid-fetch quit must emit ZoneGone"
        );
    }
}
