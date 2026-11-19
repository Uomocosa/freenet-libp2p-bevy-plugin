use crate::discovery;
use crate::identity;
use crate::testing;
use crate::time;

#[must_use]
pub fn joining_state(
    me: &str,
    room: &str,
    created_at: u64,
) -> discovery::state_machine::DiscoveryState {
    testing::DiscoveryStateBuilder::new(identity::PeerId::from(me))
        .in_room(testing::joined_room(
            room,
            created_at,
            discovery::JoinPhase::Joining {
                since: time::UnixTime::EPOCH,
            },
            Vec::new(),
        ))
        .build()
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::identity;
    use crate::time;

    use super::joining_state;

    #[test]
    fn test_usage() {
        let state = joining_state("me", "alpha", 10);
        assert_eq!(state.me.id, identity::PeerId::from("me"));
        let room = state.room.as_ref().expect("a joining room");
        assert_eq!(room.name, discovery::RoomName::from("alpha"));
        assert_eq!(room.created_at, time::UnixTime::from_secs(10));
        assert_eq!(
            room.phase,
            discovery::JoinPhase::Joining {
                since: time::UnixTime::EPOCH
            }
        );
    }
}
