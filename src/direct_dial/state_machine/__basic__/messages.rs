use bevy::prelude::Message;

use crate::direct_dial;
use crate::identity;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Tick,
    FriendAdded(identity::Peer),
    FriendRemoved(identity::PeerId),
    PeerConnected(identity::PeerId),
    PeerDisconnected(identity::PeerId),
    DialFailed(identity::PeerId),
    PeerListenAddrs(identity::Peer),
}

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Output {
    Dial(identity::Peer),
    SavePeerBook(direct_dial::PeerBook),
}
