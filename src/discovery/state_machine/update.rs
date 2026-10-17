use crate::discovery;
use discovery::state_machine::{Input, Output, State};

pub fn update(state: &mut State, input: Input, now: discovery::UnixTime) -> Vec<Output> {
    match input {
        Input::Command(command) => discovery::state_machine::handle_command(state, command, now),
        Input::NetEvent(event) => discovery::state_machine::handle_net_event(state, event, now),
        Input::LobbyUpdated(lobby) => {
            state.lobby_attached = true;
            discovery::state_machine::handle_lobby(state, lobby, now)
        }
        Input::FreenetPeers(peers) => {
            state.freenet_peers = peers;
            Vec::new()
        }
        Input::OwnAddrsChanged(addrs) => {
            state.me.addrs = addrs;
            Vec::new()
        }
        Input::Tick => discovery::state_machine::tick(state, now),
    }
}

#[cfg(test)]
mod tests {
    use super::update;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::{Input, Output, State};

    #[test]
    fn test_usage() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let room = identity::RoomName::from("r");
        let outputs = update(
            &mut state,
            Input::Command(discovery::Command::Join(room.clone())),
            discovery::UnixTime::from_secs(1),
        );
        assert_eq!(
            outputs,
            vec![Output::Notify(discovery::Event::Joined(room))]
        );
    }

    #[test]
    fn own_addrs_changed_replaces_own_addrs() {
        let mut state = State::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: vec![identity::PeerAddr::from("/ip4/1")],
        });
        let addrs = vec![identity::PeerAddr::from("/ip4/2")];
        update(
            &mut state,
            Input::OwnAddrsChanged(addrs.clone()),
            discovery::UnixTime::from_secs(1),
        );
        assert_eq!(state.me.addrs, addrs);
    }
}
