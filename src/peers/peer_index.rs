use std::collections::HashMap;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::{Commands, Entity, Resource};
use derive_more::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

use crate::identity;

#[derive(Resource, Debug, Default, Clone, Serialize, Deserialize, Deref, DerefMut)]
pub struct PeerIndex(pub HashMap<identity::PeerId, Entity>);

#[atomic_delegates]
impl PeerIndex {
    pub fn get_or_spawn(&mut self, commands: &mut Commands, id: &identity::PeerId) -> Entity {}
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::PeerIndex;
    use crate::identity;

    #[test]
    fn test_usage() {
        let mut index = PeerIndex::default();
        assert!(index.is_empty());
        index.insert(identity::PeerId::from("alice"), Entity::PLACEHOLDER);
        assert!(index.contains_key(&identity::PeerId::from("alice")));
        assert_eq!(
            index.get(&identity::PeerId::from("alice")),
            Some(&Entity::PLACEHOLDER)
        );
    }
}
