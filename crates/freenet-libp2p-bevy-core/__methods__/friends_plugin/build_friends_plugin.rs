use bevy::prelude::*;
use lele_bevy_state_machine::{StateMachinePlugin, StateMachineSystems};

use crate::direct_dial;
use crate::discovery;
use crate::p2p;
use crate::peers;
use crate::time;

pub fn build_friends_plugin(plugin: &direct_dial::FriendsPlugin, app: &mut App) {
    let config: direct_dial::Config = (**plugin).clone();
    let mut startup_error: Option<String> = None;
    let book = config
        .as_ref()
        .map_or_else(
            direct_dial::PeerBook::default,
            |path| match direct_dial::PeerBook::try_from(path.as_path()) {
                Ok(book) => book,
                Err(direct_dial::PeerBookError::Codec(_)) => {
                    if let Some(quarantined) = direct_dial::PeerBook::quarantine_corrupt_file(path)
                    {
                        tracing::error!(
                            target: "direct_dial",
                            "friends list corrupt; moved to {}",
                            quarantined.display()
                        );
                        startup_error = Some(format!(
                            "friends list was corrupt; saved a copy at {}",
                            quarantined.display()
                        ));
                    } else {
                        tracing::error!(
                            target: "direct_dial",
                            "friends list corrupt and could not be moved aside"
                        );
                        startup_error = Some(String::from(
                            "friends list is corrupt and could not be moved aside",
                        ));
                    }
                    direct_dial::PeerBook::default()
                }
                Err(error) => {
                    tracing::error!(target: "direct_dial", "friends list unreadable: {error}");
                    direct_dial::PeerBook::default()
                }
            },
        );
    app.insert_resource(config);
    app.insert_resource(direct_dial::FriendAddressError(startup_error));
    let now = time::UnixTime::now();
    app.insert_resource(direct_dial::state_machine::DialState::from_peer_book(
        &book, now,
    ));
    app.init_resource::<p2p::PendingLibp2pCommands>();
    if let Some(channel) = app.world().get_resource::<p2p::Libp2pEventChannel>() {
        app.insert_resource(direct_dial::DialEventTap(channel.subscribe()));
    }
    app.add_plugins(
        StateMachinePlugin::<direct_dial::state_machine::DialState>::default().periodic(),
    );
    app.add_systems(
        Update,
        direct_dial::bevy_systems::translate_libp2p_events_to_dial_inputs.in_set(
            StateMachineSystems::collect_inputs::<direct_dial::state_machine::DialState>(),
        ),
    );
    app.add_systems(
        Update,
        (
            direct_dial::bevy_systems::dial_outputs_to_libp2p_commands,
            direct_dial::bevy_systems::save_peer_book,
        )
            .in_set(StateMachineSystems::perform_outputs::<
                direct_dial::state_machine::DialState,
            >()),
    );
    app.add_systems(
        Update,
        direct_dial::bevy_systems::project_friends_to_entities
            .after(peers::bevy_systems::track_peer_connections)
            .before(peers::bevy_systems::despawn_orphan_peers),
    );
    app.add_message::<direct_dial::FriendAddressSubmitted>();
    app.init_resource::<direct_dial::OwnShareAddress>();
    app.init_resource::<direct_dial::FriendsView>();
    app.add_systems(Update, direct_dial::bevy_systems::update_own_share_address);
    app.add_systems(
        Update,
        direct_dial::bevy_systems::rebuild_friends_view
            .after(peers::bevy_systems::track_peer_connections)
            .after(StateMachineSystems::perform_outputs::<
                discovery::state_machine::DiscoveryState,
            >()),
    );
    app.add_systems(
        Update,
        direct_dial::bevy_systems::validate_friend_submissions.in_set(
            StateMachineSystems::collect_inputs::<direct_dial::state_machine::DialState>(),
        ),
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use lele_bevy_state_machine::testing::FrozenClock;
    use lele_bevy_state_machine::traits::StateMachine;

    use super::build_friends_plugin;
    use crate::direct_dial;
    use crate::identity;
    use crate::p2p;
    use crate::time;

    fn peer_with(name: &str, addr: &str) -> identity::Peer {
        identity::Peer {
            id: identity::PeerId::from(name),
            addrs: vec![identity::PeerAddr::from(addr)],
        }
    }

    fn app_with_book(
        book: &direct_dial::PeerBook,
    ) -> (App, tokio::sync::broadcast::Sender<p2p::Libp2pEvent>) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(p2p::Libp2pEventChannel::default());
        let frozen = time::UnixTime::from_secs(2_000_000_000);
        app.insert_resource(lele_bevy_state_machine::Now(frozen));
        app.insert_resource(FrozenClock);
        let plugin = direct_dial::FriendsPlugin::new(direct_dial::Config::from(None));
        build_friends_plugin(&plugin, &mut app);
        app.insert_resource(direct_dial::state_machine::DialState::from_peer_book(
            book, frozen,
        ));
        let tx = app.world().resource::<p2p::Libp2pEventChannel>().tx.clone();
        (app, tx)
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let plugin = direct_dial::FriendsPlugin::new(direct_dial::Config::from(None));
        build_friends_plugin(&plugin, &mut app);
        assert!(
            app.world()
                .get_resource::<direct_dial::state_machine::DialState>()
                .is_some()
        );
    }

    #[test]
    fn corrupt_book_is_quarantined_and_survives_later_saves() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("book.json");
        std::fs::write(&path, b"not json").expect("write");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let plugin = direct_dial::FriendsPlugin::new(direct_dial::Config::from(Some(path.clone())));
        build_friends_plugin(&plugin, &mut app);
        assert_eq!(
            app.world()
                .resource::<direct_dial::state_machine::DialState>()
                .entries,
            Vec::new()
        );
        assert!(
            app.world()
                .resource::<direct_dial::FriendAddressError>()
                .is_some()
        );
        let mut quarantined = path.as_os_str().to_owned();
        quarantined.push(".corrupt");
        let quarantined = std::path::PathBuf::from(quarantined);
        assert_eq!(std::fs::read(&quarantined).expect("read"), b"not json");
        let now = time::UnixTime::from_secs(10);
        app.world_mut()
            .resource_mut::<direct_dial::state_machine::DialState>()
            .update(
                direct_dial::state_machine::Input::FriendAdded(identity::Peer {
                    id: identity::PeerId::from("alice"),
                    addrs: Vec::new(),
                }),
                now,
            );
        let book = app
            .world()
            .resource::<direct_dial::state_machine::DialState>()
            .to_peer_book();
        book.save_to_file(path.as_path()).expect("save");
        assert_eq!(std::fs::read(&quarantined).expect("read"), b"not json");
        assert_eq!(
            direct_dial::PeerBook::try_from(path.as_path())
                .expect("load")
                .len(),
            1
        );
    }

    #[test]
    fn saved_friends_produce_a_dial_command() {
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(peer_with("alice", "/ip4/127.0.0.1/tcp/4001")));
        let (mut app, _tx) = app_with_book(&book);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            direct_dial::state_machine::TICK_INTERVAL,
        ));
        app.update();
        app.update();
        assert!(
            !app.world()
                .resource::<p2p::PendingLibp2pCommands>()
                .is_empty()
        );
    }

    #[test]
    fn failure_maps_to_the_right_peer() {
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(peer_with("alice", "/ip4/127.0.0.1/tcp/4001")));
        assert!(book.add_friend(peer_with("bob", "/ip4/127.0.0.1/tcp/4002")));
        let (mut app, tx) = app_with_book(&book);
        app.update();
        tx.send(p2p::Libp2pEvent::DialFailed {
            peer_id: identity::PeerId::from("alice"),
            reason: String::from("refused"),
        })
        .expect("send");
        app.update();
        let state = app
            .world()
            .resource::<direct_dial::state_machine::DialState>();
        let alice_attempts = state
            .entries
            .iter()
            .find(|entry| entry.peer.id == identity::PeerId::from("alice"))
            .map(|entry| entry.failed_attempts)
            .unwrap_or_default();
        let bob_attempts = state
            .entries
            .iter()
            .find(|entry| entry.peer.id == identity::PeerId::from("bob"))
            .map(|entry| entry.failed_attempts)
            .unwrap_or_default();
        assert_eq!(alice_attempts, 1);
        assert_eq!(bob_attempts, 0);
    }

    #[test]
    fn inbound_ephemeral_address_is_not_saved() {
        let (mut app, tx) = app_with_book(&direct_dial::PeerBook::default());
        let mallory = identity::PeerId::from("mallory");
        tx.send(p2p::Libp2pEvent::Libp2pConnectionEstablished {
            peer_id: mallory.clone(),
            kind: p2p::ConnectionKind::Tcp,
        })
        .expect("send");
        tx.send(p2p::Libp2pEvent::PeerConnected(mallory.clone()))
            .expect("send");
        app.update();
        let state = app
            .world()
            .resource::<direct_dial::state_machine::DialState>();
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0].peer.id, mallory);
        assert_eq!(state.entries[0].peer.addrs, Vec::new());
        let listen = vec![identity::PeerAddr::from("/ip4/10.0.0.5/tcp/4001")];
        tx.send(p2p::Libp2pEvent::PeerListenAddrs {
            peer_id: mallory.clone(),
            addrs: listen.clone(),
        })
        .expect("send");
        app.update();
        let state = app
            .world()
            .resource::<direct_dial::state_machine::DialState>();
        assert_eq!(state.entries.len(), 1);
        assert_eq!(state.entries[0].peer.id, mallory);
        assert_eq!(state.entries[0].peer.addrs, listen);
    }
}
