use crate::discovery;

#[must_use]
pub fn join_phase(
    state: &discovery::state_machine::DiscoveryState,
) -> Option<discovery::JoinPhase> {
    state.room.as_ref().map(|room| room.phase)
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::identity;
    use crate::testing;

    use super::join_phase;

    #[test]
    fn test_usage() {
        let state = testing::ready_state("me", "alpha", None, Vec::new(), &[]);
        assert_eq!(join_phase(&state), Some(discovery::JoinPhase::Ready));
        assert_eq!(
            join_phase(&testing::DiscoveryStateBuilder::new(identity::PeerId::from("me")).build()),
            None
        );
    }
}
