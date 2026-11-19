use bevy::prelude::App;

use crate::discovery;

pub fn add_member(app: &mut App, room: &str, member: &discovery::RoomMember) {
    let mut state = app
        .world_mut()
        .resource_mut::<discovery::state_machine::DiscoveryState>();
    if let Some(joined) = state.lobby.get_mut(&discovery::RoomName::from(room)) {
        joined
            .members
            .insert(member.peer.id.clone(), member.clone());
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    use super::add_member;

    #[test]
    fn test_usage() {
        let mut app = testing::peek_app("me", &[("alpha", 10)]);
        let member = testing::room_member("member", None, discovery::RoomMembership::Occupant, &[]);
        add_member(&mut app, "alpha", &member);
        let state = app
            .world()
            .resource::<discovery::state_machine::DiscoveryState>();
        let joined = state
            .lobby
            .get(&discovery::RoomName::from("alpha"))
            .expect("alpha");
        assert_eq!(joined.members.len(), 1);
        assert_eq!(joined.members[&identity::PeerId::from("member")], member);
    }
}
