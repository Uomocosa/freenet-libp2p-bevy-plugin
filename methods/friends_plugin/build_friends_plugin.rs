use bevy::prelude::*;

use crate::direct_dial;
use crate::p2p;
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
    app.insert_resource(book);
    app.insert_resource(direct_dial::ConnectedPeers::default());
    app.insert_resource(direct_dial::FriendAddressError(startup_error));
    if let Some(channel) = app.world().get_resource::<p2p::Libp2pEventChannel>() {
        app.insert_resource(direct_dial::FriendEventTap(channel.subscribe()));
    }
    let now = time::UnixTime::now();
    let book_now = app.world().resource::<direct_dial::PeerBook>().clone();
    app.insert_resource(direct_dial::state_machine::DialState::from_peer_book(
        &book_now, now,
    ));
    if let Some(channel) = app.world().get_resource::<p2p::Libp2pEventChannel>() {
        app.insert_resource(direct_dial::DialEventTap(channel.subscribe()));
    }
    app.add_systems(
        Update,
        (
            direct_dial::bevy_systems::drive_dial_state,
            direct_dial::bevy_systems::project_friends_to_entities,
        ),
    );
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::build_friends_plugin;
    use crate::direct_dial;
    use crate::identity;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        let plugin = direct_dial::FriendsPlugin::new(direct_dial::Config::from(None));
        build_friends_plugin(&plugin, &mut app);
        assert!(
            app.world()
                .get_resource::<direct_dial::PeerBook>()
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
        assert!(app.world().resource::<direct_dial::PeerBook>().is_empty());
        assert!(
            app.world()
                .resource::<direct_dial::FriendAddressError>()
                .is_some()
        );
        let mut quarantined = path.as_os_str().to_owned();
        quarantined.push(".corrupt");
        let quarantined = std::path::PathBuf::from(quarantined);
        assert_eq!(std::fs::read(&quarantined).expect("read"), b"not json");
        app.world_mut()
            .resource_mut::<direct_dial::PeerBook>()
            .add_friend(identity::Peer {
                id: identity::PeerId::from("alice"),
                addrs: Vec::new(),
            });
        let book = app.world().resource::<direct_dial::PeerBook>().clone();
        book.save_to_file(path.as_path()).expect("save");
        assert_eq!(std::fs::read(&quarantined).expect("read"), b"not json");
        assert_eq!(
            direct_dial::PeerBook::try_from(path.as_path())
                .expect("load")
                .len(),
            1
        );
    }
}
