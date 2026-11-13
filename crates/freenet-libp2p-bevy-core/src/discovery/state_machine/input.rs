use bevy::prelude::Message;

use crate::discovery;
use crate::identity;
use crate::p2p;

#[derive(Message, Debug, Clone, PartialEq, Eq)]
pub enum Input {
    Command(discovery::Command),
    Libp2pEvent(p2p::Libp2pEvent),
    LobbyUpdated(discovery::Lobby),
    FreenetPeers(Vec<identity::FreenetPeer>),
    OwnAddrsChanged(Vec<identity::PeerAddr>),
}

impl From<&discovery::Command> for Input {
    fn from(command: &discovery::Command) -> Self {
        Self::Command(command.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::Input;
    use crate::discovery;
    use crate::identity;

    #[test]
    fn test_usage() {
        let command = discovery::Command::Join(identity::RoomName::from("room-a"));
        assert_eq!(Input::from(&command), Input::Command(command));
    }
}
