use crate::identity;
use crate::p2p;
use crate::spectator;
use crate::time;

pub fn start_slot<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    slot: spectator::SpectateSlot,
    zone: &identity::ZoneName,
    history_prefix: p2p::HistoryKeyPrefix,
    now: time::UnixTime,
) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    let mut outputs = machine.stop_slot(slot, now);
    let lease_interval = machine.in_zone.lease_interval();
    let mut session = spectator::state_machine::SpectatorState::new(machine.game.clone());
    session.me.clone_from(&machine.me);
    let session_outputs = session.start_spectating(zone, history_prefix, now, lease_interval);
    machine.sessions.insert(slot, session);
    outputs.extend(machine.forward_outputs(slot, session_outputs, false));
    outputs
}

#[cfg(test)]
mod tests {
    use super::start_slot;
    use crate::identity;
    use crate::p2p;
    use crate::spectator;
    use crate::time;

    fn machine() -> spectator::Machine<u8> {
        let mut machine = spectator::Machine::new(identity::GameId::from("game"));
        machine.me = Some(identity::PeerId::from("me"));
        machine
    }

    fn prefix() -> p2p::HistoryKeyPrefix {
        p2p::HistoryKeyPrefix::from("prefix")
    }

    fn outputs_of(
        outputs: &[spectator::MachineOutput<u8, Vec<u8>>],
    ) -> Vec<&spectator::state_machine::SpectateOutput<u8, Vec<u8>>> {
        outputs
            .iter()
            .filter_map(|output| match output {
                spectator::MachineOutput::Session { output, .. } => Some(output),
                spectator::MachineOutput::SpectatorCountChanged(_) => None,
            })
            .collect()
    }

    #[test]
    fn test_usage() {
        let mut machine = machine();
        let outputs = start_slot(
            &mut machine,
            spectator::SpectateSlot::Preview,
            &identity::ZoneName::from("zone"),
            prefix(),
            time::UnixTime::from_secs(10),
        );
        assert_eq!(outputs_of(&outputs).len(), 4);
        assert!(
            machine
                .sessions
                .contains_key(&spectator::SpectateSlot::Preview)
        );
    }

    #[test]
    fn starting_two_slots_on_different_zones_keeps_both_sessions() {
        let mut machine = machine();
        start_slot(
            &mut machine,
            spectator::SpectateSlot::Stream,
            &identity::ZoneName::from("stream-zone"),
            prefix(),
            time::UnixTime::from_secs(10),
        );
        start_slot(
            &mut machine,
            spectator::SpectateSlot::Preview,
            &identity::ZoneName::from("preview-zone"),
            prefix(),
            time::UnixTime::from_secs(11),
        );
        assert_eq!(machine.sessions.len(), 2);
        assert_eq!(
            machine.zone_of(spectator::SpectateSlot::Stream),
            Some(&identity::ZoneName::from("stream-zone"))
        );
        assert_eq!(
            machine.zone_of(spectator::SpectateSlot::Preview),
            Some(&identity::ZoneName::from("preview-zone"))
        );
    }

    #[test]
    fn both_slots_on_the_same_zone_publish_one_lease_and_one_subscription() {
        let mut machine = machine();
        let zone = identity::ZoneName::from("shared");
        let first = start_slot(
            &mut machine,
            spectator::SpectateSlot::Preview,
            &zone,
            prefix(),
            time::UnixTime::from_secs(10),
        );
        let second = start_slot(
            &mut machine,
            spectator::SpectateSlot::Stream,
            &zone,
            prefix(),
            time::UnixTime::from_secs(11),
        );
        let leases = |outputs: &[spectator::MachineOutput<u8, Vec<u8>>]| {
            outputs_of(outputs)
                .into_iter()
                .filter(|output| {
                    matches!(
                        output,
                        spectator::state_machine::SpectateOutput::PublishLease { .. }
                    )
                })
                .count()
        };
        let subscribes = |outputs: &[spectator::MachineOutput<u8, Vec<u8>>]| {
            outputs_of(outputs)
                .into_iter()
                .filter(|output| {
                    matches!(
                        output,
                        spectator::state_machine::SpectateOutput::Subscribe(_)
                    )
                })
                .count()
        };
        assert_eq!(leases(&first), 1);
        assert_eq!(subscribes(&first), 1);
        assert_eq!(leases(&second), 0);
        assert_eq!(subscribes(&second), 0);
    }

    #[test]
    fn replacing_a_slot_zone_leaves_the_old_zone_and_keeps_the_other_slot() {
        let mut machine = machine();
        start_slot(
            &mut machine,
            spectator::SpectateSlot::Stream,
            &identity::ZoneName::from("stream-zone"),
            p2p::HistoryKeyPrefix::from("stream-prefix"),
            time::UnixTime::from_secs(10),
        );
        start_slot(
            &mut machine,
            spectator::SpectateSlot::Preview,
            &identity::ZoneName::from("preview-zone"),
            prefix(),
            time::UnixTime::from_secs(11),
        );
        let outputs = start_slot(
            &mut machine,
            spectator::SpectateSlot::Preview,
            &identity::ZoneName::from("other-zone"),
            prefix(),
            time::UnixTime::from_secs(12),
        );
        assert!(outputs_of(&outputs).iter().any(|output| matches!(
            output,
            spectator::state_machine::SpectateOutput::UnwatchZone(zone)
                if *zone == identity::ZoneName::from("preview-zone")
        )));
        assert_eq!(
            machine.zone_of(spectator::SpectateSlot::Preview),
            Some(&identity::ZoneName::from("other-zone"))
        );
        assert_eq!(
            machine.zone_of(spectator::SpectateSlot::Stream),
            Some(&identity::ZoneName::from("stream-zone"))
        );
    }
}
