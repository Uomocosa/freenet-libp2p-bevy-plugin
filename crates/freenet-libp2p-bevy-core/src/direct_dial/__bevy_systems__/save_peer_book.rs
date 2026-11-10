use bevy::prelude::*;

use crate::direct_dial;

pub fn save_peer_book(
    mut outputs: MessageReader<direct_dial::state_machine::Output>,
    config: Res<direct_dial::Config>,
) {
    let config = config.into_inner();
    for output in outputs.read() {
        if let direct_dial::state_machine::Output::SavePeerBook(book) = output
            && let Some(path) = config.as_ref()
            && let Err(error) = book.save_to_file(path)
        {
            tracing::error!(target: "direct_dial", "peer book save failed: {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::save_peer_book;
    use crate::direct_dial;
    use crate::identity;

    fn peer_with(name: &str) -> identity::Peer {
        identity::Peer {
            id: identity::PeerId::from(name),
            addrs: vec![identity::PeerAddr::from("/ip4/127.0.0.1/tcp/4001")],
        }
    }

    #[test]
    fn test_usage() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("book.json");
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<direct_dial::state_machine::Output>();
        app.insert_resource(direct_dial::Config::from(Some(path.clone())));
        app.add_systems(Update, save_peer_book);
        let mut book = direct_dial::PeerBook::default();
        assert!(book.add_friend(peer_with("alice")));
        app.world_mut()
            .resource_mut::<Messages<direct_dial::state_machine::Output>>()
            .write(direct_dial::state_machine::Output::SavePeerBook(book));
        app.update();
        let loaded = direct_dial::PeerBook::try_from(path.as_path()).expect("load");
        assert_eq!(loaded.len(), 1);
    }

    #[test]
    fn missing_path_saves_nothing() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<direct_dial::state_machine::Output>();
        app.insert_resource(direct_dial::Config::from(None));
        app.add_systems(Update, save_peer_book);
        app.world_mut()
            .resource_mut::<Messages<direct_dial::state_machine::Output>>()
            .write(direct_dial::state_machine::Output::SavePeerBook(
                direct_dial::PeerBook::default(),
            ));
        app.update();
    }
}
