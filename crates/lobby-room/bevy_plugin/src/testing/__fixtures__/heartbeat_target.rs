use crate::discovery;
use crate::identity;
use crate::time;

#[must_use]
pub fn heartbeat_target(
    room: &str,
    created_at: u64,
    membership: discovery::RoomMembership,
) -> discovery::state_machine::HeartbeatTarget {
    discovery::state_machine::HeartbeatTarget {
        room: discovery::RoomName::from(room),
        created_at: time::UnixTime::from_secs(created_at),
        addrs: vec![identity::PeerAddr::from("/ip4/1/tcp/4001")],
        display_name: identity::Username::from("someone"),
        membership,
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::time;

    use super::heartbeat_target;

    #[test]
    fn test_usage() {
        let target = heartbeat_target("room-a", 10, discovery::RoomMembership::Occupant);
        assert_eq!(target.room.as_str(), "room-a");
        assert_eq!(target.created_at, time::UnixTime::from_secs(10));
        assert_eq!(target.membership, discovery::RoomMembership::Occupant);
    }
}
