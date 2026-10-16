use std::collections::BTreeMap;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::identity;

#[derive(Debug, Clone, Default, Resource, Deref, DerefMut, Serialize, Deserialize)]
pub struct RoomRoster(pub BTreeMap<identity::RoomName, BTreeMap<[u8; 32], identity::PeerId>>);

#[rustfmt::skip]
impl RoomRoster {
    pub fn add_entry(&mut self, room: identity::RoomName, id: [u8; 32], peer: identity::PeerId) {
        self.entry(room).or_default().insert(id, peer);
    }

    pub fn remove_entry(&mut self, room: &identity::RoomName, id: [u8; 32]) -> bool {
        self.get_mut(room)
            .is_some_and(|members| members.remove(&id).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::RoomRoster;
    use crate::identity;

    #[test]
    fn test_usage() {
        let mut r = RoomRoster::default();
        let peer = identity::PeerId::from("peer");
        r.add_entry(identity::RoomName::from("room"), [1u8; 32], peer);
        assert_eq!(r.len(), 1);
    }
}
