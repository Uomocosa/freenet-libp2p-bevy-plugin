use std::collections::BTreeMap;

use crate::discovery;
use crate::p2p;

#[must_use]
pub fn history_bytes_by_room(
    rooms: &[(&str, u64)],
) -> BTreeMap<discovery::RoomName, p2p::HistoryByteCount> {
    rooms
        .iter()
        .copied()
        .map(|(room, bytes)| {
            (
                discovery::RoomName::from(room),
                p2p::HistoryByteCount(bytes),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::p2p;

    use super::history_bytes_by_room;

    #[test]
    fn test_usage() {
        let bytes = history_bytes_by_room(&[("alpha", 10)]);
        assert_eq!(
            bytes.get(&discovery::RoomName::from("alpha")),
            Some(&p2p::HistoryByteCount(10))
        );
    }
}
