use crate::discovery;
use crate::net_id;
use crate::p2p;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Command(discovery::Command),
    NetEvent(p2p::NetEvent),
    LobbyUpdated(discovery::Lobby),
    OwnAddrsChanged(Vec<net_id::PeerAddr>),
    Tick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Output {
    Notify(discovery::Event),
    NetCommand(p2p::NetCommand),
}
