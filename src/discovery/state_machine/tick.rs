use crate::discovery;
use discovery::state_machine::{Output, State};

#[must_use]
pub fn tick(state: &mut State, now: discovery::UnixTime) -> Vec<Output> {
    let mut outputs = discovery::state_machine::dial_candidates(state, now);
    let hello_due = state
        .last_hello
        .is_none_or(|last| now.since(last) >= state.timing.hello);
    if hello_due && state.room.is_some() {
        outputs.extend(discovery::state_machine::broadcast_hello(state));
        state.last_hello = Some(now);
    }
    outputs.extend(discovery::state_machine::prune_members(state, now));
    outputs
}

#[cfg(test)]
mod tests {
    use super::tick;
    use crate::discovery;
    use crate::net_id;
    use discovery::state_machine::State;

    #[test]
    fn test_usage() {
        let mut state = State::new(
            net_id::Peer {
                id: net_id::PeerId::from("me"),
                addrs: Vec::new(),
            },
            discovery::Timing::default(),
        );
        state.connected.insert(net_id::PeerId::from("a"));
        state.room = Some(discovery::Room {
            name: net_id::RoomName::from("r"),
            members: discovery::Members::new(),
        });
        let mut outputs = tick(&mut state, discovery::UnixTime::from_secs(100));
        outputs.extend(tick(&mut state, discovery::UnixTime::from_secs(101)));
        assert_eq!(state.last_hello, Some(discovery::UnixTime::from_secs(100)));
        assert_eq!(outputs.len(), 1);
    }
}
