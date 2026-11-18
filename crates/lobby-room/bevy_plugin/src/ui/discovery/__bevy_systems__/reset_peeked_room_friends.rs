use crate::ui;
use bevy::prelude::*;

use ui::discovery::PeekedRoomFriends;

pub fn reset_peeked_room_friends(mut peeked: ResMut<PeekedRoomFriends>) {
    peeked.set_if_neq(PeekedRoomFriends::default());
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::identity;
    use crate::ui;
    use bevy::prelude::*;

    use super::reset_peeked_room_friends;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(ui::discovery::PeekedRoomFriends(vec![
            core_discovery::RoomFriend {
                peer: identity::PeerId::from("alice"),
                name: identity::Username::from("Alice"),
            },
        ]));
        app.add_systems(Update, reset_peeked_room_friends);
        app.update();
        let friends = app.world().resource::<ui::discovery::PeekedRoomFriends>();
        assert_eq!(**friends, Vec::new());
    }
}
