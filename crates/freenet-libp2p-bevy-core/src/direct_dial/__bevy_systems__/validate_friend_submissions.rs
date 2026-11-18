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
