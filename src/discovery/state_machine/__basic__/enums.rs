use crate::discovery;
use crate::identity;
use crate::p2p;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Command(discovery::Command),
    Libp2pEvent(p2p::Libp2pEvent),
    LobbyUpdated(discovery::Lobby),
    FreenetPeers(Vec<identity::FreenetPeer>),
    OwnAddrsChanged(Vec<identity::PeerAddr>),
    Tick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    Notify(discovery::Event),
    Libp2pCommand(p2p::Libp2pCommand),
}
