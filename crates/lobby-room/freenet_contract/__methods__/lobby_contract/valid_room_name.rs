use crate::constants;

pub fn valid_room_name(name: &str) -> bool {
    if name.is_empty() || name.len() > constants::MAX_ROOM_NAME {
        return false;
    }
    name.chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

#[cfg(test)]
mod tests {
    use super::valid_room_name;
    use crate::constants;

    #[test]
    fn test_usage() {
        assert!(valid_room_name("room-a"));
        assert!(valid_room_name("room_A-1"));
        assert!(valid_room_name(&"a".repeat(constants::MAX_ROOM_NAME)));
        assert!(!valid_room_name(""));
        assert!(!valid_room_name("room a"));
        assert!(!valid_room_name(
            &"a".repeat(constants::MAX_ROOM_NAME.saturating_add(1))
        ));
    }
}
