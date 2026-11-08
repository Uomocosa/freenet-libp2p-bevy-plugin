use std::collections::HashMap;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::{Commands, Entity, Resource};
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::identity;

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize, Deref, DerefMut)]
pub struct RoomIndex(pub HashMap<identity::RoomName, Entity>);

#[atomic_delegates]
impl RoomIndex {
    pub fn get_or_spawn(&mut self, commands: &mut Commands, name: &identity::RoomName) -> Entity {}
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::RoomIndex;
    use crate::identity;

    #[test]
    fn test_usage() {
        let mut index = RoomIndex::default();
        assert!(index.is_empty());
        index.insert(identity::RoomName::from("lobby"), Entity::PLACEHOLDER);
        assert!(index.contains_key(&identity::RoomName::from("lobby")));
        assert_eq!(
            index.get(&identity::RoomName::from("lobby")),
            Some(&Entity::PLACEHOLDER)
        );
    }
}
