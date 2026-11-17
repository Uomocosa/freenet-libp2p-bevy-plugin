use std::collections::BTreeMap;
use std::time::Duration;

use bevy::prelude::*;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use freenet_libp2p_bevy_core::identity;
use freenet_libp2p_bevy_core::p2p;
use freenet_libp2p_bevy_core::peers;
use freenet_libp2p_bevy_core::plugin;
use freenet_libp2p_bevy_core::replica_sync;
use freenet_libp2p_bevy_core::spectator;
use freenet_libp2p_bevy_core::time;

const SETTLE: Duration = Duration::from_millis(20);
const MAX_ROUNDS: usize = 800;

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
        vec![(
            replica_sync::ReplicaSliceIndex(0),
            replica_sync::ReplicaSliceContentHash(u64::try_from(self.entries.len()).unwrap_or(0)),
        )]
    }

    fn slice_snapshot(&self, _index: replica_sync::ReplicaSliceIndex) -> Self::Snapshot {
        self.snapshot()
    }
}

fn node(game: &identity::GameId, history: Option<replica_sync::HistoryConfig>) -> App {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.add_plugins(plugin::P2PPlugin(
        plugin::Config::<u8>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled)
            .with_game(game.clone()),
    ));
    app.add_plugins(
        replica_sync::SyncPlugin::<Map>::new(game.clone())
            .with_anti_entropy_config(None)
            .with_history_config(history),
    );
    app.add_plugins(spectator::SpectatorPlugin::<Map>::new(game.clone()));
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

fn wait_ready(app: &mut App) -> Option<identity::Peer> {
    for _ in 0..MAX_ROUNDS {
        app.update();
        if let Some(peer) = ready(app) {
            return Some(peer);
        }
        std::thread::sleep(SETTLE);
    }
    None
}

fn wait_connected(
    app: &mut App,
    events: &mut tokio::sync::broadcast::Receiver<p2p::Libp2pEvent>,
    peer: &identity::PeerId,
) -> bool {
    for _ in 0..MAX_ROUNDS {
        app.update();
        while let Ok(event) = events.try_recv() {
            if let p2p::Libp2pEvent::PeerConnected(id) = event
                && &id == peer
            {
                return true;
            }
        }
        std::thread::sleep(SETTLE);
    }
    false
}

fn connect_and_seed_kad(app: &mut App, peer: &identity::Peer) -> bool {
    let mut events = app
        .world()
        .resource::<p2p::Libp2pEventChannel>()
        .subscribe();
    let sender = app.world().resource::<p2p::Libp2pCommandSender>();
    let _ = sender.send(p2p::Libp2pCommand::AddKadPeer(peer.clone()));
    let _ = sender.send(p2p::Libp2pCommand::Dial(peer.clone()));
    wait_connected(app, &mut events, &peer.id)
}

fn join_player(app: &mut App, zone: &identity::ZoneName, prefix: &p2p::HistoryKeyPrefix) {
    let zone_entity = app
        .world_mut()
        .spawn((
            zone.clone(),
            prefix.clone(),
            replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed: true,
                has_other_history_holders: false,
            },
        ))
        .id();
    let self_id = ready(app).map_or_else(|| identity::PeerId::from("me"), |peer| peer.id);
    app.world_mut().spawn((
        peers::Peer::from(self_id),
        peers::Me,
        peers::InZone(zone_entity),
    ));
}

fn history_config() -> replica_sync::HistoryConfig {
    replica_sync::HistoryConfig {
        write_interval: Duration::from_millis(80),
        record_ttl: Duration::from_secs(90),
        per_zone_bytes: p2p::HistoryByteCount::from(64u64 * 1024 * 1024),
        max_fetch_retries: 2,
    }
}

fn prepare() -> Option<(App, App, identity::ZoneName, p2p::HistoryKeyPrefix)> {
    let game = identity::GameId::from("game-spectator");
    let zone = identity::ZoneName::from("zone-spectator");
    let history = Some(history_config());
    let mut player = node(&game, history);
    let mut spectator = node(&game, None);
    let player_peer = dialable(&wait_ready(&mut player)?);
    let spectator_raw = wait_ready(&mut spectator)?;
    let spectator_peer = dialable(&spectator_raw);
    spectator
        .world_mut()
        .spawn((peers::Peer::from(spectator_raw.id), peers::Me));
    if !connect_and_seed_kad(&mut spectator, &player_peer) {
        return None;
    }
    let _ = player
        .world()
        .resource::<p2p::Libp2pCommandSender>()
        .send(p2p::Libp2pCommand::AddKadPeer(spectator_peer));
    let prefix = p2p::HistoryKeyPrefix::new(&game, &zone, time::UnixTime::now());
    join_player(&mut player, &zone, &prefix);
    player
        .world_mut()
        .resource_mut::<Map>()
        .entries
        .insert(1, 10);
    Some((player, spectator, zone, prefix))
}
fn wait_for_history(
    player: &mut App,
    events: &mut tokio::sync::broadcast::Receiver<p2p::Libp2pEvent>,
    prefix: &p2p::HistoryKeyPrefix,
) -> bool {
    for _ in 0..MAX_ROUNDS {
        player.update();
        while let Ok(event) = events.try_recv() {
            if let p2p::Libp2pEvent::HistoryRecordStored {
                id: p2p::ZoneHistoryRecordKind::Manifest(stored),
            } = event
                && &stored == prefix
            {
                return true;
            }
        }
        std::thread::sleep(SETTLE);
    }
    false
}

fn is_live_with(app: &mut App, slot: spectator::SpectateSlot, key: u8, value: u32) -> bool {
    let mut query = app.world_mut().query::<&spectator::Spectated<Map>>();
    let world = app.world();
    query.iter(world).any(|spectated| {
        spectated.slot == slot
            && spectated.freshness == spectator::SpectateFreshness::Live
            && spectated.state.entries.get(&key) == Some(&value)
    })
}

fn has_spectated(app: &mut App, slot: spectator::SpectateSlot) -> bool {
    let mut query = app.world_mut().query::<&spectator::Spectated<Map>>();
    let world = app.world();
    query.iter(world).any(|spectated| spectated.slot == slot)
}

fn wait_for_bootstrap(
    player: &mut App,
    spectator: &mut App,
    slot: spectator::SpectateSlot,
) -> bool {
    for _ in 0..MAX_ROUNDS {
        player.update();
        spectator.update();
        if is_live_with(spectator, slot, 1, 10) {
            return true;
        }
        std::thread::sleep(SETTLE);
    }
    false
}

fn wait_for_live_op(player: &mut App, spectator: &mut App, slot: spectator::SpectateSlot) -> bool {
    for round in 0..MAX_ROUNDS {
        if round % 25 == 0 {
            player
                .world_mut()
                .resource_mut::<replica_sync::PendingOps<Map>>()
                .ops
                .push(Op { key: 2, value: 20 });
        }
        player.update();
        spectator.update();
        if is_live_with(spectator, slot, 2, 20) {
            return true;
        }
        std::thread::sleep(SETTLE);
    }
    false
}

fn publish_zone_message(app: &App, topic: &identity::Topic, data: &[u8]) {
    let _ = app
        .world()
        .resource::<p2p::Libp2pCommandSender>()
        .send(p2p::Libp2pCommand::Publish {
            topic: topic.clone(),
            data: data.to_vec(),
        });
}

fn zone_message_count(app: &App, zone: &identity::ZoneName) -> usize {
    app.world()
        .resource::<p2p::PendingEvents<u8>>()
        .iter()
        .filter(|event| matches!(event, p2p::Event::ZoneMessage { zone: got, .. } if got == zone))
        .count()
}

fn wait_for_zone_message(
    player: &mut App,
    spectator: &mut App,
    zone: &identity::ZoneName,
    topic: &identity::Topic,
    data: &[u8],
) -> bool {
    for round in 0..MAX_ROUNDS {
        if round % 25 == 0 {
            publish_zone_message(player, topic, data);
        }
        player.update();
        spectator.update();
        if zone_message_count(spectator, zone) > 0 {
            return true;
        }
        std::thread::sleep(SETTLE);
    }
    false
}

fn wait_for_spectated_removed(
    player: &mut App,
    spectator: &mut App,
    slot: spectator::SpectateSlot,
) -> bool {
    for _ in 0..MAX_ROUNDS {
        player.update();
        spectator.update();
        if !has_spectated(spectator, slot) {
            return true;
        }
        std::thread::sleep(SETTLE);
    }
    false
}

fn wait_for_second_player(
    first: &mut App,
    second: &mut App,
    spectator: &mut App,
    slot: spectator::SpectateSlot,
) -> bool {
    for _ in 0..MAX_ROUNDS {
        first.update();
        second.update();
        spectator.update();
        if is_live_with(spectator, slot, 1, 10) {
            return true;
        }
        std::thread::sleep(SETTLE);
    }
    false
}

fn wait_for_second_live_op(
    first: &mut App,
    second: &mut App,
    spectator: &mut App,
    slot: spectator::SpectateSlot,
) -> bool {
    for round in 0..MAX_ROUNDS {
        if round % 25 == 0 {
            for player in [&mut *first, &mut *second] {
                player
                    .world_mut()
                    .resource_mut::<replica_sync::PendingOps<Map>>()
                    .ops
                    .push(Op { key: 2, value: 20 });
            }
        }
        first.update();
        second.update();
        spectator.update();
        if is_live_with(spectator, slot, 2, 20) {
            return true;
        }
        std::thread::sleep(SETTLE);
    }
    false
}

fn wait_for_both_counts(
    first: &mut App,
    second: &mut App,
    spectator: &mut App,
    expected: (usize, usize),
) -> bool {
    for _ in 0..MAX_ROUNDS {
        first.update();
        second.update();
        spectator.update();
        let first_count = **first.world().resource::<spectator::SpectatorCount>();
        let second_count = **second.world().resource::<spectator::SpectatorCount>();
        if first_count == expected.0 && second_count == expected.1 {
            return true;
        }
        std::thread::sleep(SETTLE);
    }
    false
}

fn wait_for_count(player: &mut App, spectator: &mut App, expected: usize) -> bool {
    for _ in 0..MAX_ROUNDS {
        player.update();
        spectator.update();
        if **player.world().resource::<spectator::SpectatorCount>() == expected {
            return true;
        }
        std::thread::sleep(SETTLE);
    }
    false
}

fn wait_for_two_counts(
    player: &mut App,
    first: &mut App,
    second: &mut App,
    expected: usize,
) -> bool {
    for _ in 0..MAX_ROUNDS {
        player.update();
        first.update();
        second.update();
        if **player.world().resource::<spectator::SpectatorCount>() == expected {
            return true;
        }
        std::thread::sleep(SETTLE);
    }
    false
}

#[tokio::test(flavor = "multi_thread")]
async fn a_spectator_sees_live_ops_and_the_count_round_trips() {
    let prepared = prepare();
    assert!(prepared.is_some(), "the two peers never prepared");
    let Some((mut player, mut spectator, zone, prefix)) = prepared else {
        return;
    };
    let mut player_events = player
        .world()
        .resource::<p2p::Libp2pEventChannel>()
        .subscribe();
    assert!(
        wait_for_history(&mut player, &mut player_events, &prefix),
        "the player never wrote its history"
    );
    spectator
        .world_mut()
        .write_message(spectator::SpectateCommand::Start {
            slot: spectator::SpectateSlot::Preview,
            zone,
            history_prefix: prefix,
        });
    assert!(
        wait_for_bootstrap(
            &mut player,
            &mut spectator,
            spectator::SpectateSlot::Preview
        ),
        "the spectator never bootstrapped from the zone history"
    );
    assert!(
        wait_for_live_op(
            &mut player,
            &mut spectator,
            spectator::SpectateSlot::Preview
        ),
        "the spectator never saw a live op"
    );
    assert!(
        wait_for_count(&mut player, &mut spectator, 1),
        "the player never counted the spectator"
    );
    spectator
        .world_mut()
        .write_message(spectator::SpectateCommand::Stop {
            slot: spectator::SpectateSlot::Preview,
        });
    assert!(
        wait_for_count(&mut player, &mut spectator, 0),
        "the player never dropped the stopped spectator"
    );
    assert!(
        !has_spectated(&mut spectator, spectator::SpectateSlot::Preview),
        "Stop must remove Spectated"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn two_spectators_on_one_zone_are_counted_separately() {
    let prepared = prepare();
    assert!(prepared.is_some(), "the two peers never prepared");
    let Some((mut player, mut first, zone, prefix)) = prepared else {
        return;
    };
    let game = identity::GameId::from("game-spectator");
    let mut second = node(&game, None);
    let second_raw = wait_ready(&mut second).expect("the second spectator never became ready");
    let second_peer = dialable(&second_raw);
    let player_peer = ready(&player).expect("the player never became ready");
    assert!(
        connect_and_seed_kad(&mut second, &dialable(&player_peer)),
        "the second spectator never connected"
    );
    let _ = player
        .world()
        .resource::<p2p::Libp2pCommandSender>()
        .send(p2p::Libp2pCommand::AddKadPeer(second_peer));
    second
        .world_mut()
        .spawn((peers::Peer::from(second_raw.id), peers::Me));
    first
        .world_mut()
        .write_message(spectator::SpectateCommand::Start {
            slot: spectator::SpectateSlot::Preview,
            zone: zone.clone(),
            history_prefix: prefix.clone(),
        });
    second
        .world_mut()
        .write_message(spectator::SpectateCommand::Start {
            slot: spectator::SpectateSlot::Preview,
            zone,
            history_prefix: prefix,
        });
    assert!(
        wait_for_two_counts(&mut player, &mut first, &mut second, 2),
        "the player never counted both spectators"
    );
    first
        .world_mut()
        .write_message(spectator::SpectateCommand::Stop {
            slot: spectator::SpectateSlot::Preview,
        });
    assert!(
        wait_for_two_counts(&mut player, &mut first, &mut second, 1),
        "the player never dropped the stopped spectator"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn a_non_member_spectator_receives_zone_messages_until_stop() {
    let prepared = prepare();
    assert!(prepared.is_some(), "the two peers never prepared");
    let Some((mut player, mut spectator, zone, prefix)) = prepared else {
        return;
    };
    let game = identity::GameId::from("game-spectator");
    let topic = identity::Topic::from_zone(&game, &zone);
    let Ok(data) = bincode::serialize(&42u8) else {
        return;
    };
    let mut player_events = player
        .world()
        .resource::<p2p::Libp2pEventChannel>()
        .subscribe();
    assert!(
        wait_for_history(&mut player, &mut player_events, &prefix),
        "the player never wrote its history"
    );
    spectator
        .world_mut()
        .write_message(spectator::SpectateCommand::Start {
            slot: spectator::SpectateSlot::Preview,
            zone: zone.clone(),
            history_prefix: prefix,
        });
    assert!(
        wait_for_bootstrap(
            &mut player,
            &mut spectator,
            spectator::SpectateSlot::Preview
        ),
        "the spectator never bootstrapped from the zone history"
    );
    assert!(
        wait_for_zone_message(&mut player, &mut spectator, &zone, &topic, &data),
        "the spectator never saw a zone message on the watched topic"
    );
    spectator
        .world_mut()
        .write_message(spectator::SpectateCommand::Stop {
            slot: spectator::SpectateSlot::Preview,
        });
    assert!(
        wait_for_spectated_removed(
            &mut player,
            &mut spectator,
            spectator::SpectateSlot::Preview
        ),
        "Stop never removed Spectated"
    );
    for _ in 0..100 {
        player.update();
        spectator.update();
        std::thread::sleep(SETTLE);
    }
    spectator
        .world_mut()
        .resource_mut::<p2p::PendingEvents<u8>>()
        .take_all();
    let mut kept_receiving = false;
    for round in 0..200 {
        if round % 25 == 0 {
            publish_zone_message(&player, &topic, &data);
        }
        player.update();
        spectator.update();
        if zone_message_count(&spectator, &zone) > 0 {
            kept_receiving = true;
            break;
        }
        std::thread::sleep(SETTLE);
    }
    assert!(
        !kept_receiving,
        "the spectator kept receiving zone messages after Stop"
    );
}

fn prepared_second_zone(
    spectator: &mut App,
    game: &identity::GameId,
    zone: &identity::ZoneName,
) -> Option<(App, p2p::HistoryKeyPrefix)> {
    let mut player = node(game, Some(history_config()));
    let player_peer = dialable(&wait_ready(&mut player)?);
    if !connect_and_seed_kad(spectator, &player_peer) {
        return None;
    }
    let spectator_peer = ready(spectator)?;
    let _ = player
        .world()
        .resource::<p2p::Libp2pCommandSender>()
        .send(p2p::Libp2pCommand::AddKadPeer(dialable(&spectator_peer)));
    let prefix = p2p::HistoryKeyPrefix::new(game, zone, time::UnixTime::now());
    join_player(&mut player, zone, &prefix);
    player
        .world_mut()
        .resource_mut::<Map>()
        .entries
        .insert(1, 10);
    let mut events = player
        .world()
        .resource::<p2p::Libp2pEventChannel>()
        .subscribe();
    wait_for_history(&mut player, &mut events, &prefix).then_some((player, prefix))
}

#[tokio::test(flavor = "multi_thread")]
async fn a_spectator_streams_one_zone_and_previews_another_at_once() {
    let prepared = prepare();
    assert!(prepared.is_some(), "the two peers never prepared");
    let Some((mut stream_player, mut spectator, stream_zone, stream_prefix)) = prepared else {
        return;
    };
    let game = identity::GameId::from("game-spectator");
    let preview_zone = identity::ZoneName::from("zone-preview");
    let Some((mut preview_player, preview_prefix)) =
        prepared_second_zone(&mut spectator, &game, &preview_zone)
    else {
        panic!("the preview player never prepared");
    };
    let mut stream_events = stream_player
        .world()
        .resource::<p2p::Libp2pEventChannel>()
        .subscribe();
    assert!(
        wait_for_history(&mut stream_player, &mut stream_events, &stream_prefix),
        "the stream player never wrote its history"
    );

    spectator
        .world_mut()
        .write_message(spectator::SpectateCommand::Start {
            slot: spectator::SpectateSlot::Stream,
            zone: stream_zone,
            history_prefix: stream_prefix,
        });
    spectator
        .world_mut()
        .write_message(spectator::SpectateCommand::Start {
            slot: spectator::SpectateSlot::Preview,
            zone: preview_zone,
            history_prefix: preview_prefix,
        });

    for slot in [
        spectator::SpectateSlot::Stream,
        spectator::SpectateSlot::Preview,
    ] {
        assert!(
            wait_for_second_player(
                &mut stream_player,
                &mut preview_player,
                &mut spectator,
                slot,
            ),
            "a slot never bootstrapped from its own zone history"
        );
        assert!(
            wait_for_second_live_op(
                &mut stream_player,
                &mut preview_player,
                &mut spectator,
                slot,
            ),
            "a slot never saw a live op in its own zone"
        );
    }
    assert!(
        wait_for_both_counts(
            &mut stream_player,
            &mut preview_player,
            &mut spectator,
            (1, 1)
        ),
        "both zones never counted the spectator"
    );

    spectator
        .world_mut()
        .write_message(spectator::SpectateCommand::Stop {
            slot: spectator::SpectateSlot::Preview,
        });

    assert!(
        wait_for_both_counts(
            &mut stream_player,
            &mut preview_player,
            &mut spectator,
            (1, 0)
        ),
        "stopping the preview never dropped its zone count"
    );
    assert!(
        !has_spectated(&mut spectator, spectator::SpectateSlot::Preview),
        "stopping the preview never removed its Spectated entity"
    );
    assert!(
        has_spectated(&mut spectator, spectator::SpectateSlot::Stream),
        "stopping the preview removed the stream Spectated entity"
    );
}
