use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::Pressed;

use crate::direct_dial;
use crate::identity;
use crate::p2p;
use direct_dial::ui::{AddFriendButton, FriendAddressInput};

pub fn handle_add_friend(
    buttons: Query<(), (Added<Pressed>, With<AddFriendButton>)>,
    input: Single<&EditableText, With<FriendAddressInput>>,
    mut book: ResMut<direct_dial::PeerBook>,
    config: Res<direct_dial::Config>,
    own: Option<Res<p2p::ReadyChannel>>,
    mut error: ResMut<direct_dial::FriendAddressError>,
) {
    let typed = input.into_inner().value().to_string();
    let config = config.into_inner();
    let own_id: Option<identity::PeerId> =
        own.and_then(|ready| ready.ready_rx.borrow().as_ref().map(|me| me.id.clone()));
    for () in &buttons {
        let trimmed = typed.trim();
        if trimmed.is_empty() {
            continue;
        }
        let Ok(peer) = identity::Peer::try_from(trimmed) else {
            **error = Some(String::from(
                "bad address: use /ip4/<host>/tcp/<port>/p2p/<id>",
            ));
            continue;
        };
        if own_id.as_ref().is_some_and(|id| *id == peer.id) {
            **error = Some(String::from("that is your own address"));
            continue;
        }
        if !book.add_friend(peer) {
            **error = Some(String::from("already a friend"));
            continue;
        }
        **error = None;
        if let Some(path) = config.as_ref()
            && let Err(save_error) = book.save_to_file(path)
        {
            tracing::error!(target: "direct_dial", "friend save failed: {save_error}");
            **error = Some(String::from("could not save the friends list"));
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::text::EditableText;
    use bevy::ui::Pressed;

    use super::handle_add_friend;
    use crate::direct_dial;
    use crate::identity;
    use crate::p2p;

    fn friend_text(id: &libp2p::PeerId) -> String {
        format!("/ip4/127.0.0.1/tcp/4001/p2p/{id}")
    }

    fn app_with(typed: &str, own_id: Option<identity::PeerId>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(direct_dial::PeerBook::default());
        app.insert_resource(direct_dial::Config::from(None));
        app.insert_resource(direct_dial::FriendAddressError::default());
        if let Some(id) = own_id {
            let ready = p2p_ready_with(id);
            app.insert_resource(ready);
        }
        app.world_mut().spawn((
            direct_dial::ui::FriendAddressInput,
            EditableText::new(typed),
        ));
        app.world_mut()
            .spawn((direct_dial::ui::AddFriendButton, Pressed));
        app.add_systems(Update, handle_add_friend);
        app
    }

    fn p2p_ready_with(id: identity::PeerId) -> p2p::ReadyChannel {
        let channel = p2p::ReadyChannel::default();
        channel.ready_tx.send_replace(Some(identity::Peer {
            id,
            addrs: Vec::new(),
        }));
        channel
    }

    #[test]
    fn test_usage() {
        let id = libp2p::PeerId::random();
        let mut app = app_with(friend_text(&id).as_str(), None);
        app.update();
        let book = app.world().resource::<direct_dial::PeerBook>();
        assert_eq!(book.len(), 1);
        assert!(
            app.world()
                .resource::<direct_dial::FriendAddressError>()
                .is_none()
        );
    }

    #[test]
    fn duplicate_friend_sets_an_error() {
        let id = libp2p::PeerId::random();
        let mut app = app_with(friend_text(&id).as_str(), None);
        app.update();
        assert_eq!(app.world().resource::<direct_dial::PeerBook>().len(), 1);
        let button = app
            .world_mut()
            .query_filtered::<Entity, With<direct_dial::ui::AddFriendButton>>()
            .single(app.world())
            .expect("add button");
        app.world_mut().entity_mut(button).remove::<Pressed>();
        app.update();
        app.world_mut().entity_mut(button).insert(Pressed);
        app.update();
        let book = app.world().resource::<direct_dial::PeerBook>();
        assert_eq!(book.len(), 1);
        assert!(
            app.world()
                .resource::<direct_dial::FriendAddressError>()
                .is_some()
        );
    }

    #[test]
    fn own_address_is_rejected() {
        let id = libp2p::PeerId::random();
        let own = identity::PeerId::from(id.to_string());
        let mut app = app_with(friend_text(&id).as_str(), Some(own));
        app.update();
        assert!(app.world().resource::<direct_dial::PeerBook>().is_empty());
        assert!(
            app.world()
                .resource::<direct_dial::FriendAddressError>()
                .is_some()
        );
    }

    #[test]
    fn bad_address_sets_an_error() {
        let mut app = app_with("not a multiaddr", None);
        app.update();
        assert!(app.world().resource::<direct_dial::PeerBook>().is_empty());
        assert!(
            app.world()
                .resource::<direct_dial::FriendAddressError>()
                .is_some()
        );
    }
}
