use bevy::prelude::*;

use crate::direct_dial;
use crate::discovery;
use crate::identity;

pub fn rebuild_lobby_rows(
    state: Option<Res<discovery::state_machine::DiscoveryState>>,
    dial: Option<Res<direct_dial::state_machine::DialState>>,
    mut rows: ResMut<discovery::LobbyRows>,
    mut friend_names: ResMut<discovery::RoomFriendNames>,
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
    let next_names = state
        .as_ref()
        .map(|state| state.room_friend_names(&friend_ids))
        .unwrap_or_default();
    friend_names.set_if_neq(discovery::RoomFriendNames(next_names));
    *rows = discovery::LobbyRows(
        state
            .map(|state| state.into_inner().lobby_rows(&friend_ids))
            .unwrap_or_default(),
    );
}
