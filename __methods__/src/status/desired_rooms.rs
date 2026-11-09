use std::collections::BTreeSet;

use crate::discovery;
use crate::identity;

#[must_use]
pub fn desired_rooms(status: &discovery::Status) -> BTreeSet<identity::RoomName> {
    let mut rooms: BTreeSet<identity::RoomName> = status.lobby.keys().cloned().collect();
    if let Some(joined) = status.room.as_ref() {
        rooms.insert(joined.name.clone());
    }
    rooms
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeMap, BTreeSet};

    use super::desired_rooms;
    use crate::discovery;
    use crate::identity;
    use crate::time;

    #[test]
    fn test_usage() {
        let room = identity::RoomName::from("room-a");
        let mut lobby = discovery::Lobby::default();
        lobby.insert(
            room.clone(),
            discovery::LobbyRoom {
                room_size: discovery::RoomSize::DEFAULT,
                created_at: time::UnixTime::from_secs(10),
                members: BTreeMap::new(),
            },
        );
        let status = discovery::Status {
            lobby,
            ..discovery::Status::default()
        };
        assert_eq!(desired_rooms(&status), BTreeSet::from([room]));
        assert!(desired_rooms(&discovery::Status::default()).is_empty());
    }
}
