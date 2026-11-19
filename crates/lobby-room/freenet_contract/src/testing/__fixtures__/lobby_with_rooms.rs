use std::collections::BTreeMap;

use common_types::UnixTime;

use crate::Lobby;
use crate::RoomName;
use crate::testing;

#[must_use]
pub fn lobby_with_rooms(count: usize, newest_at: impl Fn(usize) -> UnixTime) -> Lobby {
    Lobby::from(
        (0..count)
            .map(|index| {
                (
                    RoomName::from(format!("room-{index:03}")),
                    testing::room("peer", newest_at(index), UnixTime::from_secs(1)),
                )
            })
            .collect::<BTreeMap<_, _>>(),
    )
}

#[cfg(test)]
mod tests {
    use super::lobby_with_rooms;
    use crate::RoomName;
    use common_types::UnixTime;

    #[test]
    fn test_usage() {
        let lobby = lobby_with_rooms(2, |index| {
            UnixTime::from_secs(u64::try_from(index).unwrap_or_default())
        });
        assert_eq!(lobby.len(), 2);
        assert!(lobby.contains_key(&RoomName::from("room-001")));
    }
}
