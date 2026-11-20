use crate::identity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FanOutTarget {
    Peer(identity::PeerId),
    Game,
}
