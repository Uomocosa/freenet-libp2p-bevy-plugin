use std::time::Duration;

use crate::identity;
use crate::p2p;
use crate::spectator;
use crate::testing;
use crate::time;

#[must_use]
pub fn spectating_machine(
    slots: &[spectator::SpectateSlot],
    zone: &str,
    prefix: &str,
    lease_interval: Duration,
) -> spectator::Machine<u8> {
    let mut machine = testing::machine();
    for slot in slots {
        let mut session = spectator::state_machine::SpectatorState::new(machine.game.clone());
        session.start_spectating(
            &identity::ZoneName::from(zone),
            p2p::HistoryKeyPrefix::from(prefix),
            time::UnixTime::EPOCH,
            lease_interval,
        );
        machine.sessions.insert(*slot, session);
    }
    machine
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use crate::identity;
    use crate::spectator;

    use super::spectating_machine;

    #[test]
    fn test_usage() {
        let machine = spectating_machine(
            &[
                spectator::SpectateSlot::Preview,
                spectator::SpectateSlot::Stream,
            ],
            "shared",
            "prefix",
            Duration::from_secs(1),
        );
        assert_eq!(machine.sessions.len(), 2);
        assert_eq!(
            machine.zone_of(spectator::SpectateSlot::Stream),
            Some(&identity::ZoneName::from("shared"))
        );
    }
}
