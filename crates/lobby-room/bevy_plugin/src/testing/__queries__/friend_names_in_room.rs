use std::collections::BTreeMap;

use crate::discovery;
use crate::identity;

#[must_use]
pub fn friend_names_in_room<'a>(
    names: &'a BTreeMap<discovery::RoomName, Vec<identity::Username>>,
    room: &str,
) -> &'a [identity::Username] {
    names
        .get(&discovery::RoomName::from(room))
        .map(Vec::as_slice)
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::discovery;
    use crate::identity;

    use super::friend_names_in_room;

    #[test]
    fn test_usage() {
        let names = BTreeMap::from([(
            discovery::RoomName::from("alpha"),
            vec![identity::Username::from("Alice")],
        )]);
        assert_eq!(
            friend_names_in_room(&names, "alpha"),
            &[identity::Username::from("Alice")]
        );
        assert_eq!(friend_names_in_room(&names, "missing"), []);
    }
}
