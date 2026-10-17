use crate::discovery;
use discovery::state_machine::{HeartbeatTarget, State};

#[must_use]
pub fn heartbeat_target(state: &State) -> Option<HeartbeatTarget> {
    state.room.as_ref().map(|room| HeartbeatTarget {
        room: room.name.clone(),
        addrs: state.me.addrs.clone(),
        display_name: state.display_name.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::heartbeat_target;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/9")],
        });
        assert!(heartbeat_target(&state).is_none());
        state.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("r"),
            members: discovery::Members::new(),
        });
        let target = heartbeat_target(&state);
        assert_eq!(
            target.map(|target| target.addrs),
            Some(vec![identity::PeerAddr::from("/ip4/9")])
        );
    }
}
