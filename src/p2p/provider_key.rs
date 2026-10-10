use libp2p::kad::RecordKey;

use crate::net_id;

#[must_use]
pub fn provider_key(room: &net_id::RoomName) -> RecordKey {
    RecordKey::new(&format!("lobby/room/{}", room.as_str()))
}

#[cfg(test)]
mod tests {
    use super::provider_key;
    use crate::net_id;

    #[test]
    fn test_usage() {
        let key = provider_key(&net_id::RoomName::from("room-a"));
        assert_ne!(key.as_ref().len(), 0);
    }
}
