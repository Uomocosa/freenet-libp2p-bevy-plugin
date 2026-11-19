use crate::discovery;
use crate::identity;
use crate::testing;

#[must_use]
pub fn lobby_state(me: &str) -> discovery::state_machine::DiscoveryState {
    testing::DiscoveryStateBuilder::new(identity::PeerId::from(me))
        .lobby_fetched()
        .build()
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::identity;

    use super::lobby_state;

    #[test]
    fn test_usage() {
        let state = lobby_state("me");
        assert!(state.is_lobby_fetched);
        assert_eq!(state.me.id, identity::PeerId::from("me"));
        assert_eq!(state.lobby, discovery::Lobby::default());
    }
}
