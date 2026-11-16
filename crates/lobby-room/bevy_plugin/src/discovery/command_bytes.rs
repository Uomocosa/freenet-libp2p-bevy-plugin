use crate::discovery;

impl discovery::Command {
    #[must_use]
    pub fn to_extension_bytes(&self) -> Vec<u8> {
        bincode::serialize(self).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;

    #[test]
    fn test_usage() {
        let command = discovery::Command::Join(discovery::RoomName::from("room-1"));
        let bytes = command.to_extension_bytes();
        assert_eq!(
            bincode::deserialize::<discovery::Command>(&bytes).expect("decode"),
            command
        );
    }

    #[test]
    fn watch_round_trips() {
        let command = discovery::Command::Watch(discovery::RoomName::from("room-2"));
        let bytes = command.to_extension_bytes();
        assert_eq!(
            bincode::deserialize::<discovery::Command>(&bytes).expect("decode"),
            command
        );
    }
}
