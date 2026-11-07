use crate::direct_dial;
use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Tick,
    FriendAdded(identity::Peer),
    FriendRemoved(identity::PeerId),
    PeerConnected(identity::PeerId),
    PeerDisconnected(identity::PeerId),
    DialFailed(identity::Peer),
    PeerListenAddrs(identity::Peer),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    Dial(identity::Peer),
    SavePeerBook(direct_dial::PeerBook),
}
