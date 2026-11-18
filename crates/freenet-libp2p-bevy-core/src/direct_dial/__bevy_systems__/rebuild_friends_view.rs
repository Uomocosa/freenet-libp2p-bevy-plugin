use std::collections::BTreeSet;

use bevy::prelude::*;

use crate::direct_dial;
use crate::identity;
use crate::peers;
use crate::presence;

pub fn rebuild_friends_view(
    dial: Option<Res<direct_dial::state_machine::DialState>>,
    presence: Option<Res<presence::PeerPresence>>,
    connected_peers: Query<&peers::Peer, With<peers::Connected>>,
    mut view: ResMut<direct_dial::FriendsView>,
) {
    let friend_ids: Vec<identity::PeerId> = dial
        .map(|dial| {
            dial.entries
                .iter()
                .filter(|entry| entry.kind == direct_dial::KnownPeerKind::Friend)
                .map(|entry| entry.peer.id.clone())
                .collect()
        })
        .unwrap_or_default();
    let connected: BTreeSet<identity::PeerId> = connected_peers
        .iter()
        .map(|peer| (**peer).clone())
        .collect();
    let presence = presence
        .map(|presence| presence.into_inner().clone())
        .unwrap_or_default();
    *view = direct_dial::FriendsView::from_friend_ids(&friend_ids, &presence, &connected);
}
