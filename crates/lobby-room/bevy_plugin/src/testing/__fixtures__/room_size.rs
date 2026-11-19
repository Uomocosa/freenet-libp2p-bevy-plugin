use crate::discovery;

#[must_use]
pub fn room_size(units: u16) -> discovery::RoomSize {
    discovery::RoomSize::try_from(units).unwrap_or(discovery::RoomSize::DEFAULT)
}

#[cfg(test)]
mod tests {
    use crate::discovery;

    use super::room_size;

    #[test]
    fn test_usage() {
        assert_eq!(room_size(4).get(), 4);
        assert_eq!(room_size(0), discovery::RoomSize::DEFAULT);
    }
}
