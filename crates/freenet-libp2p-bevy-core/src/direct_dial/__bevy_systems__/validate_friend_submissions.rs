use bevy::prelude::*;

use crate::direct_dial;
use crate::identity;
use crate::p2p;

pub fn validate_friend_submissions(
    mut submitted: MessageReader<direct_dial::FriendAddressSubmitted>,
    state: Res<direct_dial::state_machine::DialState>,
    mut inputs: MessageWriter<direct_dial::state_machine::Input>,
    own: Option<Res<p2p::ReadyChannel>>,
    mut error: ResMut<direct_dial::FriendAddressError>,
) {
    let state = state.into_inner();
    let own_id: Option<identity::PeerId> =
        own.and_then(|ready| ready.ready_rx.borrow().as_ref().map(|me| me.id.clone()));
    for submitted in submitted.read() {
        let trimmed = submitted.trim();
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

    use super::validate_friend_submissions;
    use crate::direct_dial;
    use crate::identity;
    use crate::p2p;
    use crate::time;

    fn friend_text(id: &libp2p::PeerId) -> String {
        format!("/ip4/127.0.0.1/tcp/4001/p2p/{id}")
    }

    fn app_with(own_id: Option<identity::PeerId>) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<direct_dial::FriendAddressSubmitted>();
        app.add_message::<direct_dial::state_machine::Input>();
        let book = direct_dial::PeerBook::default();
        app.insert_resource(direct_dial::state_machine::DialState::from_peer_book(
            &book,
            time::UnixTime::from_secs(10),
        ));
        app.insert_resource(direct_dial::FriendAddressError::default());
        if let Some(id) = own_id {
            let channel = p2p::ReadyChannel::default();
            channel.ready_tx.send_replace(Some(identity::Peer {
                id,
                addrs: Vec::new(),
            }));
            app.insert_resource(channel);
        }
        app.add_systems(Update, validate_friend_submissions);
        app
    }

    fn submit(app: &mut App, text: &str) {
        app.world_mut()
            .resource_mut::<Messages<direct_dial::FriendAddressSubmitted>>()
            .write(direct_dial::FriendAddressSubmitted(text.to_string()));
    }

    fn drained_inputs(app: &mut App) -> Vec<direct_dial::state_machine::Input> {
        app.world_mut()
            .resource_mut::<Messages<direct_dial::state_machine::Input>>()
            .drain()
            .collect()
    }

    fn error_is(app: &App, expected: &str) -> bool {
        app.world()
            .resource::<direct_dial::FriendAddressError>()
            .as_deref()
            .is_some_and(|message| message == expected)
    }

    #[test]
    fn test_usage() {
        let id = libp2p::PeerId::random();
        let mut app = app_with(None);
        submit(&mut app, friend_text(&id).as_str());
        app.update();
        let inputs = drained_inputs(&mut app);
        assert_eq!(inputs.len(), 1);
        assert!(
            !app.world()
                .resource::<direct_dial::FriendAddressError>()
                .is_some()
        );
    }

    #[test]
    fn bad_address_sets_an_error() {
        let mut app = app_with(None);
        submit(&mut app, "not a multiaddr");
        app.update();
        assert_eq!(drained_inputs(&mut app), Vec::new());
        assert!(error_is(
            &app,
            "bad address: use /ip4/<host>/tcp/<port>/p2p/<id>"
        ));
    }

    #[test]
    fn own_address_is_rejected() {
        let id = libp2p::PeerId::random();
        let own = identity::PeerId::from(id.to_string());
        let mut app = app_with(Some(own));
        submit(&mut app, friend_text(&id).as_str());
        app.update();
        assert_eq!(drained_inputs(&mut app), Vec::new());
        assert!(error_is(&app, "that is your own address"));
    }

    #[test]
    fn duplicate_friend_sets_an_error() {
        let id = libp2p::PeerId::random();
        let mut app = app_with(None);
        submit(&mut app, friend_text(&id).as_str());
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
                next_attempt: time::Deadline::at(time::UnixTime::from_secs(10)),
            });
        submit(&mut app, friend_text(&id).as_str());
        app.update();
        assert_eq!(drained_inputs(&mut app), Vec::new());
        assert!(error_is(&app, "already a friend"));
    }

    #[test]
    fn fresh_addresses_for_a_known_friend_merge_without_an_error() {
        let id = libp2p::PeerId::random();
        let mut app = app_with(None);
        submit(&mut app, friend_text(&id).as_str());
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
                next_attempt: time::Deadline::at(time::UnixTime::from_secs(10)),
            });
        let second = format!("/ip4/127.0.0.1/tcp/5001/p2p/{id}");
        submit(&mut app, second.as_str());
        app.update();
        assert_eq!(drained_inputs(&mut app).len(), 1);
        assert!(
            !app.world()
                .resource::<direct_dial::FriendAddressError>()
                .is_some()
        );
    }

    #[test]
    fn empty_submission_writes_nothing_and_keeps_the_error() {
        let mut app = app_with(None);
        submit(&mut app, "not a multiaddr");
        app.update();
        submit(&mut app, "   ");
        app.update();
        assert_eq!(drained_inputs(&mut app), Vec::new());
        assert!(error_is(
            &app,
            "bad address: use /ip4/<host>/tcp/<port>/p2p/<id>"
        ));
    }
}
