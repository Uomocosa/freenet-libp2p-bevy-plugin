use crate::identity;
use crate::spectator;

pub fn does_any_slot_watch<Op: Clone>(
    machine: &spectator::Machine<Op>,
    zone: &identity::ZoneName,
) -> bool {
    machine
        .sessions
        .values()
        .any(|session| session.zone.as_ref() == Some(zone))
}

#[cfg(test)]
mod tests {
    use super::does_any_slot_watch;
    use crate::identity;
    use crate::p2p;
    use crate::spectator;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut machine = spectator::Machine::<u8>::new(identity::GameId::from("game"));
        assert!(!does_any_slot_watch(
            &machine,
            &identity::ZoneName::from("shared")
        ));
        let mut session =
            spectator::state_machine::SpectatorState::new(identity::GameId::from("game"));
        session.start_spectating(
            &identity::ZoneName::from("shared"),
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::EPOCH,
            spectator::state_machine::MIN_LEASE_INTERVAL,
        );
        machine
            .sessions
            .insert(spectator::SpectateSlot::Preview, session);
        assert!(does_any_slot_watch(
            &machine,
            &identity::ZoneName::from("shared")
        ));
    }
}
