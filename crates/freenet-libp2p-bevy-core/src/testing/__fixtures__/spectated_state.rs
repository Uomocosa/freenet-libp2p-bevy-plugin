use crate::identity;
use crate::spectator;

#[must_use]
pub fn spectated_state<S>(state: S) -> spectator::Spectated<S> {
    spectator::Spectated {
        slot: spectator::SpectateSlot::Preview,
        zone: identity::ZoneName::from("zone"),
        state,
        freshness: spectator::SpectateFreshness::Bootstrapping,
    }
}

#[cfg(test)]
mod tests {
    use crate::spectator;
    use crate::testing;

    use super::spectated_state;

    #[test]
    fn test_usage() {
        let spectated = spectated_state(testing::FakeMachineState::default());
        assert_eq!(spectated.slot, spectator::SpectateSlot::Preview);
        assert_eq!(
            spectated.freshness,
            spectator::SpectateFreshness::Bootstrapping
        );
    }
}
