use crate::discovery;
use discovery::state_machine::{PublishTarget, State};

#[must_use]
pub fn publish_target(state: &State) -> Option<PublishTarget> {
    state.room.as_ref().map(|room| PublishTarget {
        room: room.name.clone(),
        addrs: state.me.addrs.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::publish_target;
    use crate::discovery;
    use crate::net_id;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: vec![net_id::PeerAddr::from("/ip4/9")],
            },
            discovery::Timing::default(),
        );
        assert!(publish_target(&state).is_none());
        state.room = Some(discovery::Room {
            name: net_id::RoomName::from("r"),
            members: discovery::Members::new(),
        });
        let target = publish_target(&state);
        assert_eq!(
            target.map(|target| target.addrs),
            Some(vec![net_id::PeerAddr::from("/ip4/9")])
        );
    }
}
