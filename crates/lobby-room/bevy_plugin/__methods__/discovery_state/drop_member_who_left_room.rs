use crate::discovery;
use crate::identity;
use discovery::state_machine::{DiscoveryState, Output};

pub fn drop_member_who_left_room(
    state: &mut DiscoveryState,
    peer: &identity::PeerId,
) -> Vec<Output> {
    let removed = state
        .room
        .as_mut()
        .and_then(|room| room.members.remove(peer))
        .is_some();
    if !removed {
        return Vec::new();
    }
    tracing::info!(target: "discovery", peer = %peer.as_str(), "discovery member left");
    vec![Output::Notify(discovery::Event::RoomMembersUpdated)]
}
#[cfg(test)]
mod tests {
    use super::drop_member_who_left_room;
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    #[test]
    fn test_usage() {
        let mut state = testing::ready_state("me", "r", None, Vec::new(), &[]);
        let peer = identity::PeerId::from("a");
        assert_eq!(drop_member_who_left_room(&mut state, &peer), Vec::new());
        state.room.as_mut().expect("room").members.insert(
            peer.clone(),
            testing::room_member("a", None, discovery::RoomMembership::Occupant, &[]),
        );
        let outputs = drop_member_who_left_room(&mut state, &peer);
        assert_eq!(outputs.len(), 1);
        assert!(
            state
                .room
                .as_ref()
                .is_none_or(|room| room.members.is_empty())
        );
    }
}
