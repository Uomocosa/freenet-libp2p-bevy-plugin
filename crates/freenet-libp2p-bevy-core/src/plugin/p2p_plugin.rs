use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;
use derive_more::Deref;

use crate::p2p;
use crate::peers;
use crate::plugin;
#[cfg(feature = "testing")]
use crate::testing;
use bevy::prelude::*;
use std::sync::Mutex;

#[derive(Deref)]
pub struct P2PPlugin<Payload: p2p::traits::Message>(pub plugin::Config<Payload>);

#[rustfmt::skip]
impl<Payload: p2p::traits::Message> bevy::prelude::Plugin for P2PPlugin<Payload> {
    fn build(&self, app: &mut App) {
        let (cmd_tx, cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        let (libp2p_tx, libp2p_rx) = tokio::sync::mpsc::unbounded_channel();
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        #[cfg(feature = "testing")]
        if let Some(task_override) = app
            .world_mut()
            .remove_resource::<p2p::SwarmTaskOverride<Payload>>()
        {
            let game = self.game.clone();
            let handle = task_override.spawn_fake_swarm_task(cmd_rx, libp2p_rx, event_tx, game);
            if let Some(mut handles) = app
                .world_mut()
                .get_resource_mut::<testing::FakeTaskHandles>()
            {
                handles.swarm = Some(handle);
            }
            app.insert_resource(p2p::PendingEvents::<Payload>::default());
            app.insert_resource(p2p::PendingCommands::<Payload>::default());
            app.add_message::<p2p::Libp2pCommand>();
            app.insert_resource(p2p::ReadyChannel::default());
            app.insert_resource(p2p::Libp2pEventChannel::default());
            app.insert_resource(p2p::CommandSender(cmd_tx));
            app.insert_resource(p2p::EventReceiver(Mutex::new(Some(event_rx))));
            app.insert_resource(p2p::Libp2pCommandSender(libp2p_tx));
            app.add_systems(Update, p2p::bevy_systems::poll_p2p_events::<Payload>);
            app.add_systems(Last, p2p::bevy_systems::send_libp2p_commands);
            app.add_plugins(peers::TrackingPlugin);
            return;
        }
        let keypair = self.runner_keypair();
        let config = p2p::RunnerConfig {
            mode: self.mode,
            mdns: self.mdns,
            game: self.game.clone(),
            gossipsub: self.gossipsub,
        };
        let _runner = config.spawn_runner::<Payload>(cmd_rx, libp2p_rx, event_tx, keypair);
        app.insert_resource(p2p::PendingEvents::<Payload>::default());
        app.insert_resource(p2p::PendingCommands::<Payload>::default());
        app.add_message::<p2p::Libp2pCommand>();
        app.insert_resource(p2p::ReadyChannel::default());
        app.insert_resource(p2p::Libp2pEventChannel::default());
        app.insert_resource(p2p::CommandSender(cmd_tx));
        app.insert_resource(p2p::EventReceiver(Mutex::new(Some(event_rx))));
        app.insert_resource(p2p::Libp2pCommandSender(libp2p_tx));
        app.add_systems(Update, p2p::bevy_systems::poll_p2p_events::<Payload>);
        app.add_systems(Last, p2p::bevy_systems::send_libp2p_commands);
        app.add_plugins(peers::TrackingPlugin);
    }
}

#[atomic_delegates]
impl<Payload: p2p::traits::Message> P2PPlugin<Payload> {}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::Messages;

    use super::P2PPlugin;
    use crate::identity;
    use crate::p2p;
    use crate::plugin;
    use crate::testing;

    const SUBSCRIPTION_SETTLE: Duration = Duration::from_millis(500);
    const READY_TIMEOUT: Duration = Duration::from_secs(10);
    const ROOM_MESSAGE_TIMEOUT: Duration = Duration::from_secs(5);
    const HISTORY_EVENT_TIMEOUT: Duration = Duration::from_secs(15);

    #[tokio::test]
    async fn test_usage() {
        let mut app = testing::TestApp::new().with_plugin(P2PPlugin(plugin::Config::<
            testing::DummyMessage,
        >::new(
            p2p::TransportMode::Both,
            p2p::MdnsMode::Disabled,
        )));
        app.tick(1);
        assert!(
            app.world()
                .get_resource::<p2p::PendingEvents<testing::DummyMessage>>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<Messages<p2p::Libp2pCommand>>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<p2p::Libp2pEventChannel>()
                .is_some()
        );
        assert!(app.world().get_resource::<p2p::ReadyChannel>().is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn transport_only_three_swarms_receive_a_room_payload_exactly_once() {
        let game = identity::GameId::from("game");
        let room = identity::ZoneName::from("room-a");
        let mut hub = testing::network_node::<testing::DummyMessage>(&game);
        let mut left = testing::network_node::<testing::DummyMessage>(&game);
        let mut right = testing::network_node::<testing::DummyMessage>(&game);
        let is_room_message = |event: &p2p::Event<testing::DummyMessage>| matches!(event, p2p::Event::ZoneMessage { zone: seen, .. } if seen == &room);

        for node in [&mut hub, &mut left, &mut right] {
            node.libp2p_commands
                .send(p2p::Libp2pCommand::SubscribeZone { zone: room.clone() })
                .unwrap();
        }

        let Some(p2p::Event::Libp2p(p2p::Libp2pEvent::Ready(hub_peer))) =
            testing::wait_for_message(&mut hub.events, READY_TIMEOUT, &testing::is_ready_event)
                .await
        else {
            panic!("hub did not become ready");
        };
        let Some(p2p::Event::Libp2p(p2p::Libp2pEvent::Ready(left_peer))) =
            testing::wait_for_message(&mut left.events, READY_TIMEOUT, &testing::is_ready_event)
                .await
        else {
            panic!("left did not become ready");
        };
        let Some(p2p::Event::Libp2p(p2p::Libp2pEvent::Ready(right_peer))) =
            testing::wait_for_message(&mut right.events, READY_TIMEOUT, &testing::is_ready_event)
                .await
        else {
            panic!("right did not become ready");
        };

        left.libp2p_commands
            .send(p2p::Libp2pCommand::Dial(testing::dialable(&hub_peer)))
            .unwrap();
        right
            .libp2p_commands
            .send(p2p::Libp2pCommand::Dial(testing::dialable(&hub_peer)))
            .unwrap();

        assert!(
            testing::wait_connected_to_all(
                &mut hub,
                &[left_peer.id.clone(), right_peer.id.clone()]
            )
            .await
        );
        assert!(
            testing::wait_connected_to_all(&mut left, std::slice::from_ref(&hub_peer.id)).await
        );
        assert!(
            testing::wait_connected_to_all(&mut right, std::slice::from_ref(&hub_peer.id)).await
        );

        tokio::time::sleep(SUBSCRIPTION_SETTLE).await;

        hub.commands
            .send(p2p::Command::PublishInJoinedZone {
                payload: testing::DummyMessage(42),
            })
            .unwrap();

        for node in [&mut left, &mut right] {
            let Some(event) =
                testing::wait_for_message(&mut node.events, ROOM_MESSAGE_TIMEOUT, &is_room_message)
                    .await
            else {
                panic!("room message did not arrive");
            };
            let p2p::Event::ZoneMessage {
                zone: seen,
                from,
                payload,
            } = event
            else {
                panic!("expected a room message");
            };
            assert_eq!(seen, room);
            assert_eq!(from, hub_peer.id);
            assert_eq!(payload, testing::DummyMessage(42));
            assert!(
                testing::wait_for_message(&mut node.events, SUBSCRIPTION_SETTLE, &is_room_message)
                    .await
                    .is_none()
            );
        }

        hub.handle.abort();
        left.handle.abort();
        right.handle.abort();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn kademlia_history_record_survives_its_writer_leaving() {
        let game = identity::GameId::from("game");
        let mut writer = testing::network_node::<testing::DummyMessage>(&game);
        let mut middle = testing::network_node::<testing::DummyMessage>(&game);
        let mut reader = testing::network_node::<testing::DummyMessage>(&game);
        let Some(p2p::Event::Libp2p(p2p::Libp2pEvent::Ready(writer_peer))) =
            testing::wait_for_message(&mut writer.events, READY_TIMEOUT, &testing::is_ready_event)
                .await
        else {
            panic!("writer did not become ready");
        };
        let Some(p2p::Event::Libp2p(p2p::Libp2pEvent::Ready(middle_peer))) =
            testing::wait_for_message(&mut middle.events, READY_TIMEOUT, &testing::is_ready_event)
                .await
        else {
            panic!("middle did not become ready");
        };
        let middle_peer = testing::dialable(&middle_peer);
        let Some(p2p::Event::Libp2p(p2p::Libp2pEvent::Ready(reader_peer))) =
            testing::wait_for_message(&mut reader.events, READY_TIMEOUT, &testing::is_ready_event)
                .await
        else {
            panic!("reader did not become ready");
        };
        for node in [&writer, &reader] {
            node.libp2p_commands
                .send(p2p::Libp2pCommand::AddKadPeer(middle_peer.clone()))
                .unwrap();
            node.libp2p_commands
                .send(p2p::Libp2pCommand::Dial(middle_peer.clone()))
                .unwrap();
        }
        assert!(
            testing::wait_connected_to_all(
                &mut middle,
                &[writer_peer.id.clone(), reader_peer.id.clone()]
            )
            .await
        );
        let manifest = testing::manifest("room-generation");
        let chunk = testing::history_chunk("room-generation", "hash", 0);
        let chunk_value: Vec<u8> = (0..60 * 1024).map(|i: u32| i.to_le_bytes()[0]).collect();
        let ttl = Duration::from_secs(90);
        for (id, value) in [(&manifest, vec![7, 7, 7]), (&chunk, chunk_value.clone())] {
            let command = p2p::Libp2pCommand::PutHistoryRecord {
                id: id.clone(),
                value,
                ttl,
            };
            writer.libp2p_commands.send(command).unwrap();
            let event = testing::wait_for_message(
                &mut writer.events,
                HISTORY_EVENT_TIMEOUT,
                &testing::is_history_event,
            )
            .await
            .and_then(testing::libp2p_event)
            .expect("no history record event arrived");
            assert_eq!(
                event,
                p2p::Libp2pEvent::HistoryRecordStored { id: id.clone() }
            );
        }
        writer.handle.abort();
        for (id, value) in [(&manifest, vec![7, 7, 7]), (&chunk, chunk_value)] {
            let command = p2p::Libp2pCommand::GetHistoryRecord { id: id.clone() };
            reader.libp2p_commands.send(command).unwrap();
            let event = testing::wait_for_message(
                &mut reader.events,
                HISTORY_EVENT_TIMEOUT,
                &testing::is_history_event,
            )
            .await
            .and_then(testing::libp2p_event)
            .expect("no history record event arrived");
            assert_eq!(
                event,
                p2p::Libp2pEvent::HistoryRecordFound {
                    id: id.clone(),
                    value
                }
            );
        }
        let unknown = testing::manifest("nope");
        let command = p2p::Libp2pCommand::GetHistoryRecord {
            id: unknown.clone(),
        };
        reader.libp2p_commands.send(command).unwrap();
        let event = testing::wait_for_message(
            &mut reader.events,
            HISTORY_EVENT_TIMEOUT,
            &testing::is_history_event,
        )
        .await
        .and_then(testing::libp2p_event)
        .expect("no history record event arrived");
        let not_found = p2p::Libp2pEvent::HistoryRecordNotFound { id: unknown };
        assert_eq!(event, not_found);
        middle.handle.abort();
        reader.handle.abort();
    }
}

#[cfg(test)]
mod build_plugin_tests {
    use bevy::prelude::Plugin;

    use crate::p2p;
    use crate::plugin;
    use crate::testing;

    #[tokio::test]
    async fn test_usage() {
        let mut app = testing::TestApp::new();
        Plugin::build(
            &plugin::P2PPlugin(plugin::Config::<testing::DummyMessage>::new(
                p2p::TransportMode::Both,
                p2p::MdnsMode::Disabled,
            )),
            &mut app,
        );
        app.update();
        assert!(
            app.world()
                .get_resource::<p2p::PendingEvents<testing::DummyMessage>>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<p2p::CommandSender<testing::DummyMessage>>()
                .is_some()
        );
        assert!(
            app.world()
                .get_resource::<p2p::Libp2pCommandSender>()
                .is_some()
        );
    }
}
