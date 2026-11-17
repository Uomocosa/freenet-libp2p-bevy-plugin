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

fn prepare() -> Option<(App, App, identity::ZoneName, p2p::HistoryKeyPrefix)> {
    let game = identity::GameId::from("game-spectator");
    let zone = identity::ZoneName::from("zone-spectator");
    let history = Some(replica_sync::HistoryConfig {
        write_interval: Duration::from_millis(80),
        record_ttl: Duration::from_secs(90),
        per_zone_bytes: p2p::HistoryByteCount::from(64u64 * 1024 * 1024),
        max_fetch_retries: 2,
    });
    let mut player = node(&game, history);
    let mut spectator = node(&game, None);
    let player_peer = dialable(&wait_ready(&mut player)?);
    let spectator_peer = dialable(&wait_ready(&mut spectator)?);
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

fn wait_for_bootstrap(player: &mut App, spectator: &mut App) -> bool {
    for _ in 0..MAX_ROUNDS {
        player.update();
        spectator.update();
        if let Some(spectated) = spectator
            .world()
            .get_resource::<spectator::Spectated<Map>>()
            && spectated.freshness == spectator::SpectateFreshness::Live
            && spectated.state.entries.get(&1) == Some(&10)
        {
            return true;
        }
        std::thread::sleep(SETTLE);
    }
    false
}

fn wait_for_live_op(player: &mut App, spectator: &mut App) -> bool {
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
        if let Some(spectated) = spectator
            .world()
            .get_resource::<spectator::Spectated<Map>>()
            && spectated.state.entries.get(&2) == Some(&20)
        {
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
            zone,
            history_prefix: prefix,
        });
    assert!(
        wait_for_bootstrap(&mut player, &mut spectator),
        "the spectator never bootstrapped from the zone history"
    );
    assert!(
        wait_for_live_op(&mut player, &mut spectator),
        "the spectator never saw a live op"
    );
    assert!(
        wait_for_count(&mut player, &mut spectator, 1),
        "the player never counted the spectator"
    );
    spectator
        .world_mut()
        .write_message(spectator::SpectateCommand::Stop);
    assert!(
        wait_for_count(&mut player, &mut spectator, 0),
        "the player never dropped the stopped spectator"
    );
    assert!(
        spectator
            .world()
            .get_resource::<spectator::Spectated<Map>>()
            .is_none(),
        "Stop must remove Spectated"
    );
}
