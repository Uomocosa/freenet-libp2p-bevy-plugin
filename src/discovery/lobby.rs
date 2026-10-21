use std::collections::BTreeMap;
use std::time::Duration;

use atomic_delegate_macros::atomic_delegates;
use bevy::prelude::Reflect;
use derive_more::{Deref, DerefMut, From};
use serde::{Deserialize, Serialize};

use crate::discovery;
use crate::identity;

#[derive(
    Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Reflect, Deref, DerefMut, From,
)]
#[from(forward)]
pub struct Lobby(pub BTreeMap<identity::RoomName, discovery::Room>);

#[atomic_delegates]
impl Lobby {
    pub fn merge(&mut self, incoming: Self) {}
    pub fn drop_dead(&mut self, now: discovery::UnixTime, ttl: Duration) {}
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::discovery;
    use crate::identity;

    #[test]
    fn test_usage() {
        let heartbeat = discovery::Heartbeat {
            addrs: Vec::new(),
            updated_at: discovery::UnixTime::from_secs(1),
            display_name: None,
            presence: discovery::Presence::InRoom,
        };
        let incoming = discovery::Lobby::from([(
            identity::RoomName::from("room-a"),
            discovery::Room {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: discovery::UnixTime::from_secs(0),
                members: BTreeMap::from([(identity::PeerId::from("peer"), heartbeat)]),
            },
        )]);
        let mut lobby = discovery::Lobby::default();
        lobby.merge(incoming);
        assert_eq!(lobby.len(), 1);
        lobby.drop_dead(
            discovery::UnixTime::from_secs(500),
            discovery::HolderHeartbeatIntervals::default().ttl,
        );
        assert!(lobby.is_empty());
    }
}
