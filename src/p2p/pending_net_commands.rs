use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::p2p;
use p2p::NetCommand;

#[derive(Resource, Debug, Default, Deref, DerefMut, Serialize, Deserialize)]
pub struct PendingNetCommands(pub Vec<NetCommand>);

#[rustfmt::skip]
impl PendingNetCommands {
    pub fn take_all(&mut self) -> Vec<NetCommand> {
        self.split_off(0)
    }
}

#[cfg(test)]
mod tests {
    use super::PendingNetCommands;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut commands = PendingNetCommands::default();
        assert!(commands.is_empty());
        commands.push(p2p::NetCommand::FindRoom {
            room: identity::RoomName::from("room"),
        });
        assert_eq!(commands.len(), 1);
    }
}
