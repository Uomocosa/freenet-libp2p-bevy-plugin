use libp2p::kad::RecordKey;

use crate::identity;

#[must_use]
pub fn provider_key(room: &identity::RoomName) -> RecordKey {
    RecordKey::new(&format!("lobby/room/{}", room.as_str()))
}

#[cfg(test)]
mod tests {
    use super::provider_key;
    use crate::identity;

    #[test]
    fn test_usage() {
        let key = provider_key(&identity::RoomName::from("room-a"));
        assert_ne!(key.as_ref().len(), 0);
    }
}
