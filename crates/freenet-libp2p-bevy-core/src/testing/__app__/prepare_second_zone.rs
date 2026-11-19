use std::time::Duration;

use bevy::prelude::App;

use crate::identity;
use crate::p2p;
use crate::testing;
use crate::time;

#[must_use]
pub fn prepare_second_zone(
    spectator: &mut App,
    game: &identity::GameId,
    zone: &identity::ZoneName,
    rounds: usize,
    settle: Duration,
) -> Option<(testing::TestApp, p2p::HistoryKeyPrefix)> {
    let mut player = testing::spectator_node(game, Some(testing::history_config()));
    let player_peer = testing::dialable(&testing::wait_ready(&mut player, rounds, settle)?);
    if !testing::connect_and_seed_kad(spectator, &player_peer, rounds, settle) {
        return None;
    }
    let spectator_peer = testing::ready_peer(spectator)?;
    let _ =
        player
            .world()
            .resource::<p2p::Libp2pCommandSender>()
            .send(p2p::Libp2pCommand::AddKadPeer(testing::dialable(
                &spectator_peer,
            )));
    let prefix = p2p::HistoryKeyPrefix::new(game, zone, time::UnixTime::now());
    testing::join_player(&mut player, zone, &prefix);
    player
        .world_mut()
        .resource_mut::<testing::KeyValueMap>()
        .entries
        .insert(1, 10);
    let mut events = player
        .world()
        .resource::<p2p::Libp2pEventChannel>()
        .subscribe();
    testing::wait_for_history(&mut player, &mut events, &prefix, rounds, settle)
        .then_some((player, prefix))
}

// no test_usage necessary
