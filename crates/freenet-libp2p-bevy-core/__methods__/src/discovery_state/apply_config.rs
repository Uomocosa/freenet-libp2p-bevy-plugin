use crate::discovery;
use discovery::state_machine::DiscoveryState;

pub fn apply_config(state: &mut DiscoveryState, config: &discovery::Config) {
    state.display_name = config.display_name.clone();
    state.lease_limits = discovery::LobbyLeaseLimits {
        lease_ttl: config.lease.ttl,
        remote_clock_skew_margin: config.lease.remote_clock_skew_margin,
    };
    state.game = config.game.clone();
    state.history_size_limits = config.history_size_limits;
}

#[cfg(test)]
mod tests {
    use super::apply_config;
    use crate::discovery;
    use crate::identity;
    use discovery::state_machine::DiscoveryState;

    #[test]
    fn test_usage() {
        let mut state = DiscoveryState::new(identity::Peer {
            id: identity::PeerId::from("me"),
            addrs: Vec::new(),
        });
        let config = discovery::Config {
            display_name: identity::Username::from("alice"),
            ..discovery::Config::default()
        };
        apply_config(&mut state, &config);
        assert_eq!(state.display_name, identity::Username::from("alice"));
        assert_eq!(state.game, config.game);
    }
}
