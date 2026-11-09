use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::p2p;
use p2p::Libp2pCommand;

#[derive(Resource, Debug, Default, Deref, DerefMut, Serialize, Deserialize)]
pub struct PendingLibp2pCommands(pub Vec<Libp2pCommand>);

#[rustfmt::skip]
impl PendingLibp2pCommands {
    pub fn take_all(&mut self) -> Vec<Libp2pCommand> {
        self.split_off(0)
    }
}

#[cfg(test)]
mod tests {
    use super::PendingLibp2pCommands;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut commands = PendingLibp2pCommands::default();
        assert!(commands.is_empty());
        commands.push(p2p::Libp2pCommand::Dial(identity::Peer {
            id: identity::PeerId::from("room"),
            addrs: Vec::new(),
        }));
        assert_eq!(commands.len(), 1);
    }
}
