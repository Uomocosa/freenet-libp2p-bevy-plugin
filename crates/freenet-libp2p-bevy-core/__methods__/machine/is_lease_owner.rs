use crate::identity;
use crate::spectator;

pub fn is_lease_owner<Op: Clone>(
    machine: &spectator::Machine<Op>,
    slot: spectator::SpectateSlot,
    zone: &identity::ZoneName,
) -> bool {
    machine
        .sessions
        .iter()
        .find(|(_, session)| session.zone.as_ref() == Some(zone))
        .is_some_and(|(owner, _)| *owner == slot)
}

#[cfg(test)]
mod tests {
    use super::is_lease_owner;
    use crate::identity;
    use crate::spectator;
    use crate::testing;

    #[test]
    fn test_usage() {
        let machine = testing::spectating_machine(
            &[
                spectator::SpectateSlot::Preview,
                spectator::SpectateSlot::Stream,
            ],
            "shared",
            "prefix",
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        let zone = identity::ZoneName::from("shared");
        assert!(is_lease_owner(
            &machine,
            spectator::SpectateSlot::Preview,
            &zone
        ));
        assert!(!is_lease_owner(
            &machine,
            spectator::SpectateSlot::Stream,
            &zone
        ));
    }

    #[test]
    fn a_lone_slot_owns_its_zone() {
        let machine = testing::spectating_machine(
            &[spectator::SpectateSlot::Stream],
            "shared",
            "prefix",
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        assert!(is_lease_owner(
            &machine,
            spectator::SpectateSlot::Stream,
            &identity::ZoneName::from("shared")
        ));
    }
}
