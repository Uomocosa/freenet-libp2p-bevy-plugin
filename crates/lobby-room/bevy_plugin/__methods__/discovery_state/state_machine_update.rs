use crate::discovery;
use crate::time;

pub fn state_machine_update(
    discovery_state: &mut discovery::state_machine::DiscoveryState,
    input: discovery::state_machine::Input,
    now: time::UnixTime,
) -> Vec<discovery::state_machine::Output> {
    match input {
        discovery::state_machine::Input::Command(command) => {
            discovery_state.handle_command(command, now)
        }
        discovery::state_machine::Input::Libp2pEvent(event) => {
            discovery_state.handle_libp2p_event(event, now)
        }
        discovery::state_machine::Input::LobbyUpdated(lobby) => {
            discovery_state.is_lobby_fetched = true;
            discovery_state.handle_lobby(lobby, now)
        }
        discovery::state_machine::Input::FreenetPeers(peers) => {
            discovery_state.freenet_peers = peers;
            Vec::new()
        }
        discovery::state_machine::Input::OwnAddrsChanged(addrs) => {
            discovery_state.me.addrs = addrs;
            Vec::new()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::state_machine_update;
    use crate::discovery;
    use crate::identity;
    use crate::time;

    #[test]
    fn test_usage() {
        let me = identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        };
        let mut state = discovery::state_machine::DiscoveryState::new(me);
        let _ = state_machine_update(
            &mut state,
            discovery::state_machine::Input::FreenetPeers(Vec::new()),
            time::UnixTime::from_secs(1),
        );
    }
}
