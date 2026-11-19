use bevy::prelude::App;

use crate::discovery;
use crate::testing;

pub fn rendezvous_room(app: &mut App) {
    for (id, membership) in [
        ("member-a", discovery::RoomMembership::HistoryHolder),
        ("member-b", discovery::RoomMembership::Occupant),
        ("member-c", discovery::RoomMembership::HistoryHolder),
        ("member-d", discovery::RoomMembership::Occupant),
        ("member-e", discovery::RoomMembership::HistoryHolder),
    ] {
        let member = testing::room_member(id, None, membership, &["/ip4/127.0.0.1/tcp/4001"]);
        testing::add_member(app, "alpha", &member);
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::testing;

    use super::rendezvous_room;

    #[test]
    fn test_usage() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        rendezvous_room(&mut app);
        let state = app
            .world()
            .resource::<discovery::state_machine::DiscoveryState>();
        let joined = state
            .lobby
            .get(&discovery::RoomName::from("alpha"))
            .expect("alpha");
        assert_eq!(joined.members.len(), 5);
    }
}
