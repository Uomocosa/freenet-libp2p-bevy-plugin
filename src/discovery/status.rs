use std::collections::BTreeSet;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Resource;

use crate::discovery;
use crate::identity;
use crate::p2p;
use crate::room_sync;

#[derive(Resource, Debug, Clone, Default, PartialEq, Eq)]
pub struct Status {
    pub lobby: discovery::Lobby,
    pub room: Option<discovery::JoinedRoom>,
    pub connected: BTreeSet<identity::PeerId>,
    pub freenet_peers: Vec<identity::FreenetPeer>,
    pub is_lobby_attached: bool,
    pub self_peer_id: Option<identity::PeerId>,
}

#[rustfmt::skip]
impl Status {
    #[must_use]
    pub fn link_status(&self, peer: &identity::PeerId) -> discovery::LinkStatus {
        if self.connected.contains(peer) {
            discovery::LinkStatus::Connected
        } else {
            discovery::LinkStatus::Lost
        }
    }

    pub fn connected_members(&self) -> impl Iterator<Item = &identity::PeerId> {
        let room = self.room.as_ref();
        self.connected
            .iter()
            .filter(move |peer| room.is_some_and(|room| room.members.contains_key(*peer)))
    }
}

#[atomic_delegates]
impl Status {
    pub(crate) fn room_prefix_and_lobby_status(
        status: Option<&Self>,
        game: &identity::GameId,
        room_name: &identity::RoomName,
    ) -> (
        p2p::HistoryKeyPrefix,
        room_sync::state_machine::LobbyRoomStatus,
    ) {
    }
}

// no test_usage necessary
