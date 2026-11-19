use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::App;
use derive_more::Deref;

use crate::direct_dial;
use crate::discovery;
use crate::p2p;
use crate::peers;
use crate::replica_sync;
use bevy::prelude::*;
use discovery::Config;
use lele_bevy_state_machine::{StateMachinePlugin, StateMachineSystems};

#[derive(Deref)]
pub struct Plugin(pub Config);

impl Plugin {
    #[must_use]
    pub const fn new(config: Config) -> Self {
        Self(config)
    }
}

impl bevy::prelude::Plugin for Plugin {
    fn build(&self, app: &mut App) {
        let config = (**self).clone();
        app.insert_resource(config);
        app.add_message::<discovery::Command>();
        app.add_message::<discovery::Event>();
        app.add_message::<p2p::Libp2pCommand>();
        if let Some(channel) = app.world().get_resource::<p2p::Libp2pEventChannel>() {
            app.insert_resource(discovery::Libp2pEventTap(channel.subscribe()));
        }
        app.add_plugins(
            StateMachinePlugin::<discovery::state_machine::DiscoveryState>::default()
                .periodic()
                .also_update_in(Last)
                .input_from::<discovery::Command>()
                .output_into::<discovery::Event>()
                .output_into::<p2p::Libp2pCommand>(),
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
                .before(peers::bevy_systems::despawn_orphan_zones)
                .before(direct_dial::bevy_systems::project_friends_to_entities)
                .in_set(replica_sync::ReplicaSyncSet::Input),
        );
        app.add_systems(
            Update,
            discovery::bevy_systems::leave_failed_room.after(replica_sync::ReplicaSyncSet::Drive),
        );
        app.init_resource::<discovery::LobbyRows>();
        app.init_resource::<discovery::RoomFriendNames>();
        app.init_resource::<discovery::RoomRoster>();
        Self::publish_presence(app);
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
            discovery::bevy_systems::send_quit_on_exit.in_set(
                StateMachineSystems::collect_inputs::<discovery::state_machine::DiscoveryState>(),
            ),
        );
        app.add_systems(
            Last,
            (discovery::bevy_systems::publish_heartbeat_targets,).in_set(
                StateMachineSystems::perform_outputs::<discovery::state_machine::DiscoveryState>(),
            ),
        );
        app.configure_sets(
            Last,
            StateMachineSystems::perform_outputs::<discovery::state_machine::DiscoveryState>()
                .before(p2p::bevy_systems::send_libp2p_commands),
        );
        self.spawn_lobby_task(app);
    }
}

#[atomic_delegates]
impl Plugin {
    pub(crate) fn publish_presence(app: &mut App) {}
    pub(crate) fn spawn_lobby_task(&self, app: &mut App) {}
    pub(crate) fn swarm_channels(app: &App) -> Option<p2p::SwarmChannels> {}
}
// no test_usage necessary

#[cfg(test)]
mod build_plugin_tests {
    use bevy::prelude::Plugin;
    use bevy::prelude::*;

    use crate::discovery;
    use crate::identity;
    use crate::p2p;
    use crate::peers;
    use crate::presence;
    use crate::replica_sync;
    use crate::testing;
    use crate::time;

    #[tokio::test]
    async fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let plugin = discovery::Plugin::new(discovery::Config::default());
        Plugin::build(&plugin, &mut app);
        assert!(app.world().get_resource::<discovery::Config>().is_some());
    }

    #[tokio::test]
    async fn joining_a_room_subscribes_the_swarm_to_the_room_topic() {
        let mut app = testing::discovery_app_with_swarm();
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
        let room = discovery::RoomName::from("room-a");
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
                p2p::Libp2pCommand::SubscribeZone { zone: joined } if *joined == *room
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
        let mut app = testing::discovery_app_with_swarm();
        let room = discovery::RoomName::from("room-a");
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
    fn a_history_store_measurement_reaches_the_sender_exactly_once() {
        let mut app = testing::discovery_app_with_swarm();
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
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        app.insert_resource(p2p::Libp2pCommandSender(tx));
        app.world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .write(discovery::state_machine::Input::Libp2pEvent(
                p2p::Libp2pEvent::HistoryRecordStored {
                    id: p2p::ZoneHistoryRecordKind::Manifest(p2p::HistoryKeyPrefix::from("abc")),
                },
            ));
        app.update();
        app.update();
        app.update();
        let mut measurements = 0;
        while let Ok(command) = rx.try_recv() {
            if command == p2p::Libp2pCommand::MeasureHistoryStore {
                measurements += 1;
            }
        }
        assert_eq!(measurements, 1, "one output must produce one send");
    }

    #[test]
    fn quit_on_app_exit_produces_departed_heartbeat_and_leave_handshake() {
        let mut app = testing::discovery_app_with_swarm();
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
                discovery::Command::Create(discovery::RoomName::from("room-a")),
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

    #[test]
    fn an_occupant_is_published_with_its_room_and_display_name() {
        let mut app = testing::discovery_app_with_swarm();
        let room = discovery::RoomName::from("room-7");
        let peer = identity::PeerId::from("bob");
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            room.clone(),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members: std::collections::BTreeMap::from([(
                    peer.clone(),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: peer.clone(),
                            addrs: Vec::new(),
                        },
                        display_name: Some(identity::Username::from("Bob")),
                        updated_at: time::UnixTime::EPOCH,
                        membership: discovery::RoomMembership::Occupant,
                    },
                )]),
            },
        );
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .with_lobby(lobby)
                .build(),
        );
        app.update();
        assert_eq!(
            app.world().resource::<presence::PeerPresence>().get(&peer),
            Some(&presence::Whereabouts {
                zone: identity::Zone::Restricted(identity::ZoneName::clone(&room)),
                display_name: Some(identity::Username::from("Bob")),
            })
        );
    }

    #[test]
    fn a_history_holder_publishes_no_presence() {
        let mut app = testing::discovery_app_with_swarm();
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            discovery::RoomName::from("room-7"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::EPOCH,
                members: std::collections::BTreeMap::from([(
                    identity::PeerId::from("bob"),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: identity::PeerId::from("bob"),
                            addrs: Vec::new(),
                        },
                        display_name: Some(identity::Username::from("Bob")),
                        updated_at: time::UnixTime::EPOCH,
                        membership: discovery::RoomMembership::HistoryHolder,
                    },
                )]),
            },
        );
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .with_lobby(lobby)
                .build(),
        );
        app.update();
        assert!(app.world().resource::<presence::PeerPresence>().is_empty());
    }

    #[test]
    fn a_peer_in_two_rooms_takes_the_first_room_by_name() {
        let mut app = testing::discovery_app_with_swarm();
        let peer = identity::PeerId::from("bob");
        let mut lobby = discovery::Lobby::default();
        for room in ["room-b", "room-a"] {
            lobby.insert(
                discovery::RoomName::from(room),
                discovery::Room {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at: time::UnixTime::EPOCH,
                    members: std::collections::BTreeMap::from([(
                        peer.clone(),
                        discovery::RoomMember {
                            peer: identity::Peer {
                                id: peer.clone(),
                                addrs: Vec::new(),
                            },
                            display_name: Some(identity::Username::from("Bob")),
                            updated_at: time::UnixTime::EPOCH,
                            membership: discovery::RoomMembership::Occupant,
                        },
                    )]),
                },
            );
        }
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .with_lobby(lobby)
                .build(),
        );
        app.update();
        let zone = app
            .world()
            .resource::<presence::PeerPresence>()
            .get(&peer)
            .map(|whereabouts| whereabouts.zone.clone());
        assert_eq!(
            zone,
            Some(identity::Zone::Restricted(identity::ZoneName::from(
                "room-a"
            )))
        );
    }

    #[test]
    fn the_plugin_projects_a_room_prefix_and_liveness() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<AppExit>();
        app.add_plugins(discovery::Plugin::new(discovery::Config::default()));
        let game = identity::GameId::from("test");
        app.insert_resource(replica_sync::SyncGame(game.clone()));
        let room = discovery::RoomName::from("room-a");
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            room.clone(),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::from_secs(10),
                members: std::collections::BTreeMap::from([(
                    identity::PeerId::from("alice"),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: identity::PeerId::from("alice"),
                            addrs: Vec::new(),
                        },
                        display_name: None,
                        updated_at: time::UnixTime::from_secs(10),
                        membership: discovery::RoomMembership::Occupant,
                    },
                )]),
            },
        );
        let state = testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .with_lobby(lobby)
            .build();
        let (expected_prefix, expected_liveness) =
            discovery::state_machine::DiscoveryState::room_prefix_and_lobby_status(
                Some(&state),
                &game,
                &room,
            );
        app.insert_resource(state);
        app.update();
        let mut rooms = app.world_mut().query::<(Entity, &identity::ZoneName)>();
        let room_entity = rooms
            .iter(app.world())
            .find(|(_, name)| **name == *room)
            .map(|(entity, _)| entity)
            .expect("room entity");
        assert_eq!(
            app.world().get::<p2p::HistoryKeyPrefix>(room_entity),
            Some(&expected_prefix)
        );
        assert_eq!(
            app.world()
                .get::<replica_sync::state_machine::ZoneLiveness>(room_entity),
            Some(&expected_liveness)
        );
    }

    #[test]
    fn a_stale_room_entity_is_not_reused() {
        let mut app = testing::discovery_app_with_swarm();
        app.insert_resource(replica_sync::SyncGame(identity::GameId::from("game")));
        let room = discovery::RoomName::from("room-a");
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            room.clone(),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::from_secs(10),
                members: std::collections::BTreeMap::from([(
                    identity::PeerId::from("alice"),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: identity::PeerId::from("alice"),
                            addrs: Vec::new(),
                        },
                        display_name: None,
                        updated_at: time::UnixTime::from_secs(10),
                        membership: discovery::RoomMembership::Occupant,
                    },
                )]),
            },
        );
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .with_lobby(lobby)
                .build(),
        );
        app.update();
        let old = {
            let mut rooms = app.world_mut().query::<(Entity, &identity::ZoneName)>();
            rooms
                .iter(app.world())
                .find(|(_, name)| **name == *room)
                .map(|(entity, _)| entity)
                .expect("room entity")
        };
        app.world_mut().despawn(old);
        app.update();
        let fresh = {
            let mut rooms = app.world_mut().query::<(Entity, &identity::ZoneName)>();
            rooms
                .iter(app.world())
                .find(|(_, name)| **name == *room)
                .map(|(entity, _)| entity)
                .expect("fresh room entity")
        };
        assert_ne!(fresh, old);
        assert!(app.world().get_entity(fresh).is_ok());
    }

    #[test]
    fn a_stale_peer_entity_is_not_reused() {
        let mut app = testing::discovery_app_with_swarm();
        app.insert_resource(replica_sync::SyncGame(identity::GameId::from("game")));
        let room = discovery::RoomName::from("room-a");
        let alice = identity::PeerId::from("alice");
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            room,
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::from_secs(10),
                members: std::collections::BTreeMap::from([(
                    alice.clone(),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: alice.clone(),
                            addrs: Vec::new(),
                        },
                        display_name: None,
                        updated_at: time::UnixTime::from_secs(10),
                        membership: discovery::RoomMembership::Occupant,
                    },
                )]),
            },
        );
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .with_lobby(lobby)
                .build(),
        );
        app.update();
        let old = {
            let mut peers_found = app.world_mut().query::<(Entity, &peers::Peer)>();
            peers_found
                .iter(app.world())
                .find(|(_, peer)| ***peer == alice)
                .map(|(entity, _)| entity)
                .expect("peer entity")
        };
        app.world_mut().despawn(old);
        app.update();
        let fresh = {
            let mut peers_found = app.world_mut().query::<(Entity, &peers::Peer)>();
            peers_found
                .iter(app.world())
                .find(|(_, peer)| ***peer == alice)
                .map(|(entity, _)| entity)
                .expect("fresh peer entity")
        };
        assert_ne!(fresh, old);
        assert!(app.world().get_entity(fresh).is_ok());
    }

    #[test]
    fn a_holder_of_two_rooms_links_both_without_churn() {
        let mut app = testing::discovery_app_with_swarm();
        app.insert_resource(replica_sync::SyncGame(identity::GameId::from("game")));
        let alice = identity::PeerId::from("alice");
        let mut lobby = discovery::Lobby::default();
        for room in ["room-a", "room-b"] {
            lobby.insert(
                discovery::RoomName::from(room),
                discovery::Room {
                    room_size: discovery::RoomSize::DEFAULT,
                    created_at: time::UnixTime::from_secs(10),
                    members: std::collections::BTreeMap::from([(
                        alice.clone(),
                        discovery::RoomMember {
                            peer: identity::Peer {
                                id: alice.clone(),
                                addrs: Vec::new(),
                            },
                            display_name: None,
                            updated_at: time::UnixTime::from_secs(10),
                            membership: discovery::RoomMembership::HistoryHolder,
                        },
                    )]),
                },
            );
        }
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .with_lobby(lobby)
                .build(),
        );
        app.update();
        let hold_link_count = |app: &mut App| {
            app.world_mut()
                .query_filtered::<Entity, With<peers::HistoryHold>>()
                .iter(app.world())
                .count()
        };
        assert_eq!(hold_link_count(&mut app), 2);
        let alice_entity = {
            let mut peers_found = app.world_mut().query::<(Entity, &peers::Peer)>();
            peers_found
                .iter(app.world())
                .find(|(_, peer)| ***peer == alice)
                .map(|(entity, _)| entity)
                .expect("alice peer entity")
        };
        assert_eq!(
            app.world()
                .get::<peers::HeldZones>(alice_entity)
                .expect("held zones")
                .len(),
            2
        );
        let room_entities: Vec<Entity> = {
            let mut rooms = app.world_mut().query::<(Entity, &identity::ZoneName)>();
            rooms.iter(app.world()).map(|(entity, _)| entity).collect()
        };
        assert_eq!(room_entities.len(), 2);
        for room in room_entities {
            assert!(
                app.world()
                    .get::<peers::HistoryHolders>(room)
                    .is_some_and(|holders| holders.len() == 1)
            );
        }
        for _ in 0..4 {
            app.update();
            assert_eq!(hold_link_count(&mut app), 2);
        }
    }

    #[test]
    fn a_departed_holder_frees_its_links_and_the_room_is_swept() {
        let mut app = testing::discovery_app_with_swarm();
        app.add_plugins(peers::TrackingPlugin);
        app.insert_resource(replica_sync::SyncGame(identity::GameId::from("game")));
        let room = discovery::RoomName::from("room-a");
        let alice = identity::PeerId::from("alice");
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            room.clone(),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::from_secs(10),
                members: std::collections::BTreeMap::from([(
                    alice.clone(),
                    discovery::RoomMember {
                        peer: identity::Peer {
                            id: alice.clone(),
                            addrs: Vec::new(),
                        },
                        display_name: None,
                        updated_at: time::UnixTime::from_secs(10),
                        membership: discovery::RoomMembership::HistoryHolder,
                    },
                )]),
            },
        );
        app.insert_resource(
            testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
                .with_lobby(lobby)
                .build(),
        );
        app.update();
        let hold_link_count = |app: &mut App| {
            app.world_mut()
                .query_filtered::<Entity, With<peers::HistoryHold>>()
                .iter(app.world())
                .count()
        };
        assert_eq!(hold_link_count(&mut app), 1);
        let room_entity = {
            let mut rooms = app.world_mut().query::<(Entity, &identity::ZoneName)>();
            rooms
                .iter(app.world())
                .find(|(_, name)| **name == *room)
                .map(|(entity, _)| entity)
                .expect("room entity")
        };
        assert!(app.world().get_entity(room_entity).is_ok());
        {
            let mut state = app
                .world_mut()
                .resource_mut::<discovery::state_machine::DiscoveryState>();
            state
                .lobby
                .get_mut(&room)
                .expect("room in lobby")
                .members
                .get_mut(&alice)
                .expect("alice listed")
                .membership = discovery::RoomMembership::Departed;
            state.lobby.remove(&room);
        }
        app.update();
        app.update();
        assert_eq!(hold_link_count(&mut app), 0);
        assert!(app.world().get_entity(room_entity).is_err());
        let mut rooms = app.world_mut().query::<(Entity, &identity::ZoneName)>();
        assert_eq!(rooms.iter(app.world()).count(), 0);
    }
}
