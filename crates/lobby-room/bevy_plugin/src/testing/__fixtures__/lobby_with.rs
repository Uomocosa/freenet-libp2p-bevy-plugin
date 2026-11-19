use std::collections::BTreeMap;

use crate::discovery;

#[must_use]
pub fn lobby_with(rooms: Vec<(&str, discovery::Room)>) -> discovery::Lobby {
    rooms
        .into_iter()
        .map(|(name, room)| (discovery::RoomName::from(name), room))
        .collect::<BTreeMap<_, _>>()
        .into()
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::testing;

    use super::lobby_with;

    #[test]
    fn test_usage() {
        let lobby = lobby_with(vec![(
            "alpha",
            testing::room(discovery::RoomSize::DEFAULT, 0, Vec::new()),
        )]);
        assert_eq!(lobby.len(), 1);
        assert!(lobby.contains_key(&discovery::RoomName::from("alpha")));
    }
}
