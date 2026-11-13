use bevy::prelude::*;
use lele_bevy_state_machine::{StateMachinePlugin, StateMachineSystems};

use crate::direct_dial;
use crate::discovery;
use crate::p2p;
use crate::peers;
use crate::room_sync;

pub fn build_plugin(plugin: &discovery::Plugin, app: &mut App) {
    let config = (**plugin).clone();
    app.insert_resource(config);
    app.add_message::<discovery::Command>();
    app.add_message::<discovery::Event>();
    app.init_resource::<p2p::PendingLibp2pCommands>();
    if let Some(channel) = app.world().get_resource::<p2p::Libp2pEventChannel>() {
        app.insert_resource(discovery::Libp2pEventTap(channel.subscribe()));
    }
    app.add_plugins(
        StateMachinePlugin::<discovery::state_machine::DiscoveryState>::default()
            .periodic()
            .also_update_in(Last)
            .input_from::<discovery::Command>()
            .output_into::<discovery::Event>(),
    );
    app.add_systems(
        Update,
        (
            discovery::bevy_systems::start_discovery_when_swarm_ready.run_if(not(
                resource_exists::<discovery::state_machine::DiscoveryState>,
            )),
            discovery::bevy_systems::translate_libp2p_events_to_discovery_inputs,
            discovery::bevy_systems::lobby_updates_to_discovery_inputs,
            discovery::bevy_systems::own_addrs_to_discovery_inputs,
        )
            .in_set(StateMachineSystems::collect_inputs::<
                discovery::state_machine::DiscoveryState,
            >()),
    );
    app.add_systems(
        Update,
        (
            discovery::bevy_systems::follow_room.after(
                lele_bevy_state_machine::bevy_systems::forward_output::<
                    discovery::state_machine::DiscoveryState,
                    discovery::Event,
                >,
            ),
            discovery::bevy_systems::discovery_outputs_to_libp2p_commands,
            discovery::bevy_systems::publish_heartbeat_targets,
        )
            .in_set(StateMachineSystems::perform_outputs::<
                discovery::state_machine::DiscoveryState,
            >()),
    );
    app.add_systems(
        Update,
        discovery::bevy_systems::project_status_to_entities
            .run_if(resource_exists::<discovery::state_machine::DiscoveryState>)
            .after(peers::bevy_systems::track_peer_connections)
            .before(peers::bevy_systems::despawn_orphan_peers)
            .before(peers::bevy_systems::despawn_orphan_rooms)
            .before(direct_dial::bevy_systems::project_friends_to_entities)
            .in_set(room_sync::RoomSyncSet::Input),
    );
    app.add_systems(
        Update,
        discovery::bevy_systems::leave_failed_room.after(room_sync::RoomSyncSet::Drive),
    );
    app.init_resource::<discovery::LobbyRows>();
    app.init_resource::<discovery::RoomRoster>();
    app.add_systems(
        Update,
        (
            discovery::bevy_systems::rebuild_lobby_rows,
            discovery::bevy_systems::rebuild_room_roster,
        )
            .run_if(resource_exists::<discovery::state_machine::DiscoveryState>)
            .after(StateMachineSystems::perform_outputs::<
                discovery::state_machine::DiscoveryState,
            >())
            .after(peers::bevy_systems::track_peer_connections),
    );
    app.add_systems(
        Last,
        discovery::bevy_systems::send_quit_on_exit.in_set(StateMachineSystems::collect_inputs::<
            discovery::state_machine::DiscoveryState,
        >()),
    );
    app.add_systems(
        Last,
        (
            discovery::bevy_systems::discovery_outputs_to_libp2p_commands,
            discovery::bevy_systems::publish_heartbeat_targets,
        )
            .in_set(StateMachineSystems::perform_outputs::<
                discovery::state_machine::DiscoveryState,
            >()),
    );
    app.add_systems(
        Last,
        discovery::bevy_systems::flush_pending_libp2p_commands.after(
            StateMachineSystems::perform_outputs::<discovery::state_machine::DiscoveryState>(),
        ),
    );
    plugin.spawn_lobby_task(app);
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use lele_bevy_state_machine::Now;
    use lele_bevy_state_machine::testing::FrozenClock;

    use super::build_plugin;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::time;

    #[tokio::test]
    async fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let plugin = discovery::Plugin::new(discovery::Config::default());
        build_plugin(&plugin, &mut app);
        assert!(app.world().get_resource::<discovery::Config>().is_some());
    }

    fn app_with_swarm() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::ReadyChannel::default());
        app.insert_resource(p2p::Libp2pEventChannel::default());
        let (cmd_tx, _cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        app.insert_resource(p2p::Libp2pCommandSender(cmd_tx));
        let frozen = time::UnixTime::from_secs(2_000_000_000);
        app.insert_resource(Now(frozen));
        app.insert_resource(FrozenClock);
        let plugin = discovery::Plugin::new(discovery::Config::default());
        build_plugin(&plugin, &mut app);
        app
    }

    #[tokio::test]
    async fn joining_a_room_subscribes_the_swarm_to_the_room_topic() {
        let mut app = app_with_swarm();
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        app.insert_resource(p2p::Libp2pCommandSender(tx));
        let peer = identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/1.2.3.4/tcp/4001")],
        };
        app.world()
            .resource::<p2p::ReadyChannel>()
            .ready_tx
            .send_replace(Some(peer));
        app.update();
        app.update();
        let room = identity::RoomName::from("room-a");
        app.world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .write(discovery::state_machine::Input::Command(
                discovery::Command::Create(room.clone()),
            ));
        app.update();
        app.update();
        let mut saw_subscribe = false;
        while let Ok(command) = rx.try_recv() {
            if matches!(
                &command,
                p2p::Libp2pCommand::SubscribeRoom { room: joined } if *joined == room
            ) {
                saw_subscribe = true;
            }
        }
        assert!(
            saw_subscribe,
            "joining must subscribe the swarm to the room topic"
        );
    }

    #[test]
    fn inputs_before_swarm_ready_are_processed_after_state_appears() {
        let mut app = app_with_swarm();
        let room = identity::RoomName::from("room-a");
        app.world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .write(discovery::state_machine::Input::Command(
                discovery::Command::Create(room.clone()),
            ));
        app.update();
        assert!(
            app.world()
                .get_resource::<discovery::state_machine::DiscoveryState>()
                .is_none()
        );
        let peer = identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/1.2.3.4/tcp/4001")],
        };
        app.world()
            .resource::<p2p::ReadyChannel>()
            .ready_tx
            .send_replace(Some(peer));
        app.update();
        app.update();
        let state = app
            .world()
            .resource::<discovery::state_machine::DiscoveryState>();
        assert_eq!(state.room.as_ref().map(|joined| &joined.name), Some(&room));
    }

    #[test]
    fn quit_on_app_exit_produces_departed_heartbeat_and_leave_handshake() {
        let mut app = app_with_swarm();
        let peer = identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/1.2.3.4/tcp/4001")],
        };
        app.world()
            .resource::<p2p::ReadyChannel>()
            .ready_tx
            .send_replace(Some(peer));
        app.update();
        app.update();
        app.world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .write(discovery::state_machine::Input::Command(
                discovery::Command::Create(identity::RoomName::from("room-a")),
            ));
        app.update();
        app.update();
        assert!(
            app.world()
                .resource::<discovery::state_machine::DiscoveryState>()
                .room
                .is_some()
        );
        let alice = identity::PeerId::from("alice");
        app.world_mut()
            .resource_mut::<discovery::state_machine::DiscoveryState>()
            .connected
            .insert(alice.clone());
        let (cmd_tx, mut cmd_rx) = tokio::sync::mpsc::unbounded_channel();
        app.insert_resource(p2p::Libp2pCommandSender(cmd_tx));
        app.world_mut()
            .resource_mut::<p2p::PendingLibp2pCommands>()
            .take_all();
        app.world_mut().write_message(AppExit::Success);
        app.update();
        let mut saw_handshake = false;
        while let Ok(command) = cmd_rx.try_recv() {
            if matches!(
                &command,
                p2p::Libp2pCommand::PeerExchange { peer_id, .. } if *peer_id == alice
            ) {
                saw_handshake = true;
            }
        }
        assert!(
            saw_handshake,
            "quit leaves a handshake for the connected peer"
        );
        let targets = app
            .world()
            .resource::<discovery::HeartbeatTargetSender>()
            .borrow()
            .clone();
        assert!(!targets.is_empty(), "quit publishes heartbeat targets");
        assert!(
            targets
                .iter()
                .all(|target| target.membership == discovery::RoomMembership::Departed),
            "quit marks every heartbeat target Departed"
        );
    }
}
