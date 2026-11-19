use crate::discovery;
use crate::time;

#[must_use]
pub const fn held_room_history(
    created_at: u64,
    left_at: u64,
) -> discovery::state_machine::HeldRoomHistory {
    discovery::state_machine::HeldRoomHistory {
        created_at: time::UnixTime::from_secs(created_at),
        left_at: time::UnixTime::from_secs(left_at),
    }
}

#[cfg(test)]
mod tests {
    use crate::time;

    use super::held_room_history;

    #[test]
    fn test_usage() {
        let history = held_room_history(10, 20);
        assert_eq!(history.created_at, time::UnixTime::from_secs(10));
        assert_eq!(history.left_at, time::UnixTime::from_secs(20));
    }
}
