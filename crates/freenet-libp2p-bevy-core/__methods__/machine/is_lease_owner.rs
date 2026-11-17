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
    use crate::p2p;
    use crate::spectator;
    use crate::time;

    fn machine_watching(both: bool) -> spectator::Machine<u8> {
        let mut machine = spectator::Machine::new(identity::GameId::from("game"));
        for slot in [
            spectator::SpectateSlot::Preview,
            spectator::SpectateSlot::Stream,
        ] {
            if both || slot == spectator::SpectateSlot::Stream {
                let mut session =
                    spectator::state_machine::SpectatorState::new(identity::GameId::from("game"));
                session.start_spectating(
                    &identity::ZoneName::from("shared"),
                    p2p::HistoryKeyPrefix::from("prefix"),
                    time::UnixTime::EPOCH,
                    spectator::state_machine::MIN_LEASE_INTERVAL,
                );
                machine.sessions.insert(slot, session);
            }
        }
        machine
    }

    #[test]
    fn test_usage() {
        let machine = machine_watching(true);
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
        let machine = machine_watching(false);
        assert!(is_lease_owner(
            &machine,
            spectator::SpectateSlot::Stream,
            &identity::ZoneName::from("shared")
        ));
    }
}
