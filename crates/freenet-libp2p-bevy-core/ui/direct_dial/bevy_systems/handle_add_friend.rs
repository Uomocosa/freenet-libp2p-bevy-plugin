use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::Pressed;

use crate::direct_dial;
use crate::identity;
use crate::p2p;
use crate::ui;
use ui::direct_dial::{AddFriendButton, FriendAddressInput};

pub fn handle_add_friend(
    buttons: Query<(), (Added<Pressed>, With<AddFriendButton>)>,
    input: Single<&EditableText, With<FriendAddressInput>>,
    state: Res<direct_dial::state_machine::DialState>,
    mut inputs: MessageWriter<direct_dial::state_machine::Input>,
    own: Option<Res<p2p::ReadyChannel>>,
    mut error: ResMut<direct_dial::FriendAddressError>,
) {
    let state = state.into_inner();
    let typed = input.into_inner().value().to_string();
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
        if state.is_friend(&peer.id) {
            let same = state
                .entries
                .iter()
                .find(|entry| entry.peer.id == peer.id)
                .is_some_and(|entry| entry.peer.addrs == peer.addrs);
            if same {
                **error = Some(String::from("already a friend"));
                continue;
            }
        }
        **error = None;
        inputs.write(direct_dial::state_machine::Input::FriendAdded(peer));
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
    use crate::time;
    use crate::ui;

    fn friend_text(id: &libp2p::PeerId) -> String {
        format!("/ip4/127.0.0.1/tcp/4001/p2p/{id}")
    }

    fn app_with(typed: &str, own_id: Option<identity::PeerId>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<direct_dial::state_machine::Input>();
        let book = direct_dial::PeerBook::default();
        app.insert_resource(direct_dial::state_machine::DialState::from_peer_book(
            &book,
            time::UnixTime::from_secs(10),
        ));
        app.insert_resource(direct_dial::FriendAddressError::default());
        if let Some(id) = own_id {
            let ready = p2p_ready_with(id);
            app.insert_resource(ready);
        }
        app.world_mut().spawn((
            ui::direct_dial::FriendAddressInput,
            EditableText::new(typed),
        ));
        app.world_mut()
            .spawn((ui::direct_dial::AddFriendButton, Pressed));
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

    fn drained_inputs(app: &mut App) -> Vec<direct_dial::state_machine::Input> {
        app.world_mut()
            .resource_mut::<Messages<direct_dial::state_machine::Input>>()
            .drain()
            .collect()
    }

    #[test]
    fn test_usage() {
        let id = libp2p::PeerId::random();
        let mut app = app_with(friend_text(&id).as_str(), None);
        app.update();
        let inputs = drained_inputs(&mut app);
        assert_eq!(inputs.len(), 1);
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
        assert_eq!(drained_inputs(&mut app).len(), 1);
        let peer = identity::Peer::try_from(friend_text(&id).as_str()).expect("peer");
        app.world_mut()
            .resource_mut::<direct_dial::state_machine::DialState>()
            .entries
            .push(direct_dial::state_machine::DialKnownPeer {
                peer,
                kind: direct_dial::KnownPeerKind::Friend,
                failed_attempts: 0,
                next_attempt_at: time::UnixTime::from_secs(10),
            });
        let button = app
            .world_mut()
            .query_filtered::<Entity, With<ui::direct_dial::AddFriendButton>>()
            .single(app.world())
            .expect("add button");
        app.world_mut().entity_mut(button).remove::<Pressed>();
        app.update();
        app.world_mut().entity_mut(button).insert(Pressed);
        app.update();
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
        assert_eq!(drained_inputs(&mut app), Vec::new());
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
        assert_eq!(drained_inputs(&mut app), Vec::new());
        assert!(
            app.world()
                .resource::<direct_dial::FriendAddressError>()
                .is_some()
        );
    }

    #[test]
    fn fresh_addresses_for_a_known_friend_merge_without_an_error() {
        let id = libp2p::PeerId::random();
        let mut app = app_with(friend_text(&id).as_str(), None);
        app.update();
        assert_eq!(drained_inputs(&mut app).len(), 1);
        let peer = identity::Peer::try_from(friend_text(&id).as_str()).expect("peer");
        app.world_mut()
            .resource_mut::<direct_dial::state_machine::DialState>()
            .entries
            .push(direct_dial::state_machine::DialKnownPeer {
                peer,
                kind: direct_dial::KnownPeerKind::Friend,
                failed_attempts: 0,
                next_attempt_at: time::UnixTime::from_secs(10),
            });
        let second = format!("/ip4/127.0.0.1/tcp/5001/p2p/{id}");
        {
            let world = app.world_mut();
            let mut inputs = world
                .query_filtered::<&mut EditableText, With<ui::direct_dial::FriendAddressInput>>();
            if let Ok(mut input) = inputs.single_mut(world) {
                *input = EditableText::new(second);
            } else {
                panic!("friend input missing");
            }
        }
        app.update();
        let button = app
            .world_mut()
            .query_filtered::<Entity, With<ui::direct_dial::AddFriendButton>>()
            .single(app.world())
            .expect("add button");
        app.world_mut().entity_mut(button).remove::<Pressed>();
        app.update();
        app.world_mut().entity_mut(button).insert(Pressed);
        app.update();
        let inputs = drained_inputs(&mut app);
        assert_ne!(inputs, Vec::new());
        assert!(
            app.world()
                .resource::<direct_dial::FriendAddressError>()
                .is_none()
        );
    }
}
