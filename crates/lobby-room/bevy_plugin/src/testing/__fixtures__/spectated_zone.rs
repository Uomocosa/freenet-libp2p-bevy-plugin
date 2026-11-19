use crate::identity;
use crate::spectator;
use crate::testing;

#[must_use]
pub fn spectated_zone(
    slot: spectator::SpectateSlot,
    zone: &str,
    freshness: spectator::SpectateFreshness,
) -> spectator::Spectated<testing::FakeReplica<u8, u8>> {
    spectator::Spectated {
        slot,
        zone: identity::ZoneName::from(zone),
        state: testing::FakeReplica::default(),
        freshness,
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::spectator;

    use super::spectated_zone;

    #[test]
    fn test_usage() {
        let spectated = spectated_zone(
            spectator::SpectateSlot::Stream,
            "beta",
            spectator::SpectateFreshness::Live,
        );
        assert_eq!(spectated.slot, spectator::SpectateSlot::Stream);
        assert_eq!(spectated.zone, identity::ZoneName::from("beta"));
        assert_eq!(spectated.freshness, spectator::SpectateFreshness::Live);
    }
}
