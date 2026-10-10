use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::p2p;
use p2p::NetCommand;

#[derive(Resource, Debug, Default, Deref, DerefMut, Serialize, Deserialize)]
pub struct Outbox(pub Vec<NetCommand>);

#[rustfmt::skip]
impl Outbox {
    pub fn take_all(&mut self) -> Vec<NetCommand> {
        self.split_off(0)
    }
}

#[cfg(test)]
mod tests {
    use super::Outbox;
    use crate::net_id;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut commands = Outbox::default();
        assert!(commands.is_empty());
        commands.push(p2p::NetCommand::FindRoom {
            room: net_id::RoomName::from("room"),
        });
        assert_eq!(commands.len(), 1);
    }
}
