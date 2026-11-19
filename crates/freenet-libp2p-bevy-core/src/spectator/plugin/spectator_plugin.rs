use std::marker::PhantomData;
use std::sync::Mutex;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::*;

use crate::identity;
use crate::p2p;
use crate::replica_sync;
use crate::spectator;

pub struct SpectatorPlugin<S: replica_sync::traits::ReplicatedState> {
    pub game: identity::GameId,
    pub marker: PhantomData<S>,
}

impl<S: replica_sync::traits::ReplicatedState> SpectatorPlugin<S> {
    #[must_use]
    pub const fn new(game: identity::GameId) -> Self {
        Self {
            game,
            marker: PhantomData,
        }
    }
}

#[rustfmt::skip]
impl<S: replica_sync::traits::ReplicatedState> bevy::prelude::Plugin for SpectatorPlugin<S> {
    fn build(&self, app: &mut App) {
        let Some(libp2p_events) = app
            .world()
            .get_resource::<p2p::Libp2pEventChannel>()
            .map(p2p::Libp2pEventChannel::subscribe)
        else {
            tracing::error!(target: "spectator", "spectator: p2p Libp2pEventChannel missing");
            return;
        };
        app.add_message::<spectator::SpectateCommand>();
        app.insert_resource(spectator::SpectatorGame(self.game.clone()));
        app.insert_resource(spectator::SpectatorEvents(Mutex::new(libp2p_events)));
        app.insert_resource(spectator::Machine::<S::Op>::new(self.game.clone()));
        app.insert_resource(spectator::SpectatorInputs::<S::Op, Vec<u8>>(Vec::new()));
        app.init_resource::<spectator::HistoryBootstrap<Vec<u8>>>();
        app.init_resource::<spectator::SpectatorCount>();
        app.add_systems(
            Update,
            (
                spectator::bevy_systems::sync_local_peer::<S>,
                spectator::bevy_systems::manage_spectated::<S>,
                spectator::bevy_systems::route_spectate_commands::<S>,
                spectator::bevy_systems::route_topic_messages::<S>,
                spectator::bevy_systems::count_spectators::<S>,
                spectator::bevy_systems::drive::<S>,
            )
                .chain()
                .in_set(replica_sync::ReplicaSyncSet::Drive),
        );
    }
}

#[atomic_delegates]
impl<S: replica_sync::traits::ReplicatedState> SpectatorPlugin<S> {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::SpectatorPlugin;
    use crate::identity;
    use crate::p2p;
    use crate::peers;
    use crate::plugin;
    use crate::replica_sync;
    use crate::spectator;
    use crate::testing;

    const SETTLE: Duration = Duration::from_millis(20);
    const MAX_ROUNDS: usize = 800;
    const PUSH_EVERY: usize = 25;

    #[tokio::test]
    async fn test_usage() {
        let mut app = testing::TestApp::new().with_plugin(plugin::P2PPlugin(
            plugin::Config::<u8>::new(p2p::TransportMode::Both, p2p::MdnsMode::Disabled),
        ));
        app.add_plugins(SpectatorPlugin::<testing::FakeReplica<u8, u8>>::new(
            identity::GameId::from("test"),
        ));
        app.update();
        assert!(
            app.world()
                .get_resource::<spectator::Machine<u8>>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<spectator::SpectatorCount>()
                .is_some()
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_spectator_sees_live_ops_and_the_count_round_trips() {
        let prepared = testing::spectator_pair(MAX_ROUNDS, SETTLE);
        assert!(prepared.is_some(), "the two peers never prepared");
        let Some((mut player, mut spectator, zone, prefix)) = prepared else {
            return;
        };
        let mut player_events = player
            .world()
            .resource::<p2p::Libp2pEventChannel>()
            .subscribe();
        assert!(
            testing::wait_for_history(&mut player, &mut player_events, &prefix, MAX_ROUNDS, SETTLE),
            "the player never wrote its history"
        );
        testing::spectate(
            &mut spectator,
            spectator::SpectateSlot::Preview,
            zone,
            prefix,
        );
        assert!(
            testing::wait_until(MAX_ROUNDS, SETTLE, |_| {
                player.update();
                spectator.update();
                testing::is_live_with(&mut spectator, spectator::SpectateSlot::Preview, 1, 10)
            }),
            "the spectator never bootstrapped from the zone history"
        );
        assert!(
            testing::wait_until(MAX_ROUNDS, SETTLE, |round| {
                if round % PUSH_EVERY == 0 {
                    player
                        .world_mut()
                        .resource_mut::<replica_sync::PendingOps<testing::KeyValueMap>>()
                        .ops
                        .push(testing::KeyValueOp { key: 2, value: 20 });
                }
                player.update();
                spectator.update();
                testing::is_live_with(&mut spectator, spectator::SpectateSlot::Preview, 2, 20)
            }),
            "the spectator never saw a live op"
        );
        assert!(
            testing::wait_until(MAX_ROUNDS, SETTLE, |_| {
                player.update();
                spectator.update();
                testing::spectator_count(&player) == 1
            }),
            "the player never counted the spectator"
        );
        testing::stop_spectating(&mut spectator, spectator::SpectateSlot::Preview);
        assert!(
            testing::wait_until(MAX_ROUNDS, SETTLE, |_| {
                player.update();
                spectator.update();
                testing::spectator_count(&player) == 0
            }),
            "the player never dropped the stopped spectator"
        );
        assert!(
            !testing::has_spectated(&mut spectator, spectator::SpectateSlot::Preview),
            "Stop must remove Spectated"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn two_spectators_on_one_zone_are_counted_separately() {
        let prepared = testing::spectator_pair(MAX_ROUNDS, SETTLE);
        assert!(prepared.is_some(), "the two peers never prepared");
        let Some((mut player, mut first, zone, prefix)) = prepared else {
            return;
        };
        let game = identity::GameId::from("game-spectator");
        let mut second = testing::spectator_node(&game, None);
        let second_raw = testing::wait_ready(&mut second, MAX_ROUNDS, SETTLE)
            .expect("the second spectator never became ready");
        let second_peer = testing::dialable(&second_raw);
        let player_peer = testing::ready_peer(&player).expect("the player never became ready");
        assert!(
            testing::connect_and_seed_kad(
                &mut second,
                &testing::dialable(&player_peer),
                MAX_ROUNDS,
                SETTLE
            ),
            "the second spectator never connected"
        );
        let _ = player
            .world()
            .resource::<p2p::Libp2pCommandSender>()
            .send(p2p::Libp2pCommand::AddKadPeer(second_peer));
        second
            .world_mut()
            .spawn((peers::Peer::from(second_raw.id), peers::Me));
        testing::spectate(
            &mut first,
            spectator::SpectateSlot::Preview,
            zone.clone(),
            prefix.clone(),
        );
        testing::spectate(&mut second, spectator::SpectateSlot::Preview, zone, prefix);
        assert!(
            testing::wait_until(MAX_ROUNDS, SETTLE, |_| {
                player.update();
                first.update();
                second.update();
                testing::spectator_count(&player) == 2
            }),
            "the player never counted both spectators"
        );
        testing::stop_spectating(&mut first, spectator::SpectateSlot::Preview);
        assert!(
            testing::wait_until(MAX_ROUNDS, SETTLE, |_| {
                player.update();
                first.update();
                second.update();
                testing::spectator_count(&player) == 1
            }),
            "the player never dropped the stopped spectator"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn a_non_member_spectator_receives_zone_messages_until_stop() {
        let prepared = testing::spectator_pair(MAX_ROUNDS, SETTLE);
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
            testing::wait_for_history(&mut player, &mut player_events, &prefix, MAX_ROUNDS, SETTLE),
            "the player never wrote its history"
        );
        testing::spectate(
            &mut spectator,
            spectator::SpectateSlot::Preview,
            zone.clone(),
            prefix,
        );
        assert!(
            testing::wait_until(MAX_ROUNDS, SETTLE, |_| {
                player.update();
                spectator.update();
                testing::is_live_with(&mut spectator, spectator::SpectateSlot::Preview, 1, 10)
            }),
            "the spectator never bootstrapped from the zone history"
        );
        assert!(
            testing::wait_until(MAX_ROUNDS, SETTLE, |round| {
                if round % PUSH_EVERY == 0 {
                    testing::publish_zone_message(&player, &topic, &data);
                }
                player.update();
                spectator.update();
                testing::zone_message_count::<u8>(&spectator, &zone) > 0
            }),
            "the spectator never saw a zone message on the watched topic"
        );
        testing::stop_spectating(&mut spectator, spectator::SpectateSlot::Preview);
        assert!(
            testing::wait_until(MAX_ROUNDS, SETTLE, |_| {
                player.update();
                spectator.update();
                !testing::has_spectated(&mut spectator, spectator::SpectateSlot::Preview)
            }),
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
            if round % PUSH_EVERY == 0 {
                testing::publish_zone_message(&player, &topic, &data);
            }
            player.update();
            spectator.update();
            if testing::zone_message_count::<u8>(&spectator, &zone) > 0 {
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

    #[tokio::test(flavor = "multi_thread")]
    async fn a_spectator_streams_one_zone_and_previews_another_at_once() {
        let prepared = testing::spectator_pair(MAX_ROUNDS, SETTLE);
        assert!(prepared.is_some(), "the two peers never prepared");
        let Some((mut stream_player, mut spectator, stream_zone, stream_prefix)) = prepared else {
            return;
        };
        let game = identity::GameId::from("game-spectator");
        let preview_zone = identity::ZoneName::from("zone-preview");
        let Some((mut preview_player, preview_prefix)) =
            testing::prepare_second_zone(&mut spectator, &game, &preview_zone, MAX_ROUNDS, SETTLE)
        else {
            panic!("the preview player never prepared");
        };
        let mut stream_events = stream_player
            .world()
            .resource::<p2p::Libp2pEventChannel>()
            .subscribe();
        assert!(
            testing::wait_for_history(
                &mut stream_player,
                &mut stream_events,
                &stream_prefix,
                MAX_ROUNDS,
                SETTLE
            ),
            "the stream player never wrote its history"
        );

        testing::spectate(
            &mut spectator,
            spectator::SpectateSlot::Stream,
            stream_zone,
            stream_prefix,
        );
        testing::spectate(
            &mut spectator,
            spectator::SpectateSlot::Preview,
            preview_zone,
            preview_prefix,
        );

        for slot in [
            spectator::SpectateSlot::Stream,
            spectator::SpectateSlot::Preview,
        ] {
            assert!(
                testing::wait_until(MAX_ROUNDS, SETTLE, |_| {
                    stream_player.update();
                    preview_player.update();
                    spectator.update();
                    testing::is_live_with(&mut spectator, slot, 1, 10)
                }),
                "a slot never bootstrapped from its own zone history"
            );
            assert!(
                testing::wait_until(MAX_ROUNDS, SETTLE, |round| {
                    if round % PUSH_EVERY == 0 {
                        for player in [&mut stream_player, &mut preview_player] {
                            player
                                .world_mut()
                                .resource_mut::<replica_sync::PendingOps<testing::KeyValueMap>>()
                                .ops
                                .push(testing::KeyValueOp { key: 2, value: 20 });
                        }
                    }
                    stream_player.update();
                    preview_player.update();
                    spectator.update();
                    testing::is_live_with(&mut spectator, slot, 2, 20)
                }),
                "a slot never saw a live op in its own zone"
            );
        }
        assert!(
            testing::wait_until(MAX_ROUNDS, SETTLE, |_| {
                stream_player.update();
                preview_player.update();
                spectator.update();
                testing::spectator_count(&stream_player) == 1
                    && testing::spectator_count(&preview_player) == 1
            }),
            "both zones never counted the spectator"
        );

        testing::stop_spectating(&mut spectator, spectator::SpectateSlot::Preview);

        assert!(
            testing::wait_until(MAX_ROUNDS, SETTLE, |_| {
                stream_player.update();
                preview_player.update();
                spectator.update();
                testing::spectator_count(&stream_player) == 1
                    && testing::spectator_count(&preview_player) == 0
            }),
            "stopping the preview never dropped its zone count"
        );
        assert!(
            !testing::has_spectated(&mut spectator, spectator::SpectateSlot::Preview),
            "stopping the preview never removed its Spectated entity"
        );
        assert!(
            testing::has_spectated(&mut spectator, spectator::SpectateSlot::Stream),
            "stopping the preview removed the stream Spectated entity"
        );
    }
}
