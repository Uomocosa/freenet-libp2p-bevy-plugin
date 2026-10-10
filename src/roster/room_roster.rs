use std::collections::BTreeMap;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::net_id;

#[derive(Debug, Clone, Default, Resource, Deref, DerefMut, Serialize, Deserialize)]
pub struct RoomRoster(pub BTreeMap<net_id::RoomName, BTreeMap<[u8; 32], net_id::PeerId>>);

#[rustfmt::skip]
impl RoomRoster {
    pub fn add_entry(&mut self, room: net_id::RoomName, id: [u8; 32], peer: net_id::PeerId) {
        self.entry(room).or_default().insert(id, peer);
    }

    pub fn remove_entry(&mut self, room: &net_id::RoomName, id: [u8; 32]) -> bool {
        self.get_mut(room)
            .is_some_and(|members| members.remove(&id).is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::RoomRoster;
    use crate::net_id;

    #[test]
    fn test_usage() {
        let mut r = RoomRoster::default();
        let peer = net_id::PeerId::from("peer");
        r.add_entry(net_id::RoomName::from("room"), [1u8; 32], peer);
        assert_eq!(r.len(), 1);
    }
}
