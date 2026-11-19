use std::time::Duration;

use crate::identity;
use crate::p2p;
use crate::peers;
use crate::testing;
use crate::time;

#[must_use]
pub fn spectator_pair(
    rounds: usize,
    settle: Duration,
) -> Option<(
    testing::TestApp,
    testing::TestApp,
    identity::ZoneName,
    p2p::HistoryKeyPrefix,
)> {
    let game = identity::GameId::from("game-spectator");
    let zone = identity::ZoneName::from("zone-spectator");
    let history = Some(testing::history_config());
    let mut player = testing::spectator_node(&game, history);
    let mut spectator = testing::spectator_node(&game, None);
    let player_peer = testing::dialable(&testing::wait_ready(&mut player, rounds, settle)?);
    let spectator_raw = testing::wait_ready(&mut spectator, rounds, settle)?;
    let spectator_peer = testing::dialable(&spectator_raw);
    spectator
        .world_mut()
        .spawn((peers::Peer::from(spectator_raw.id), peers::Me));
    if !testing::connect_and_seed_kad(&mut spectator, &player_peer, rounds, settle) {
        return None;
    }
    let _ = player
        .world()
        .resource::<p2p::Libp2pCommandSender>()
        .send(p2p::Libp2pCommand::AddKadPeer(spectator_peer));
    let prefix = p2p::HistoryKeyPrefix::new(&game, &zone, time::UnixTime::now());
    testing::join_player(&mut player, &zone, &prefix);
    player
        .world_mut()
        .resource_mut::<testing::KeyValueMap>()
        .entries
        .insert(1, 10);
    Some((player, spectator, zone, prefix))
}

// no test_usage necessary
