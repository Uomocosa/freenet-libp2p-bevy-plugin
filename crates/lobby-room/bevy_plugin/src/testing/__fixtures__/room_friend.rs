use crate::discovery;
use crate::identity;

#[must_use]
pub fn room_friend(peer: &str, name: &str) -> discovery::RoomFriend {
    discovery::RoomFriend {
        peer: identity::PeerId::from(peer),
        name: identity::Username::from(name),
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;

    use super::room_friend;

    #[test]
    fn test_usage() {
        let friend = room_friend("alice", "Alice");
        assert_eq!(friend.peer, identity::PeerId::from("alice"));
        assert_eq!(friend.name, identity::Username::from("Alice"));
    }
}
