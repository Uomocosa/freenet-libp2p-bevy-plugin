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
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut machine = testing::machine();
        machine.me = Some(identity::PeerId::from("me"));
        let outputs = start_slot(
            &mut machine,
            spectator::SpectateSlot::Preview,
            &identity::ZoneName::from("zone"),
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::from_secs(10),
        );
        assert_eq!(testing::session_outputs(&outputs).len(), 4);
        assert!(
            machine
                .sessions
                .contains_key(&spectator::SpectateSlot::Preview)
        );
    }

    #[test]
    fn starting_two_slots_on_different_zones_keeps_both_sessions() {
        let mut machine = testing::machine();
        machine.me = Some(identity::PeerId::from("me"));
        start_slot(
            &mut machine,
            spectator::SpectateSlot::Stream,
            &identity::ZoneName::from("stream-zone"),
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::from_secs(10),
        );
        start_slot(
            &mut machine,
            spectator::SpectateSlot::Preview,
            &identity::ZoneName::from("preview-zone"),
            p2p::HistoryKeyPrefix::from("prefix"),
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
        let mut machine = testing::machine();
        machine.me = Some(identity::PeerId::from("me"));
        let zone = identity::ZoneName::from("shared");
        let first = start_slot(
            &mut machine,
            spectator::SpectateSlot::Preview,
            &zone,
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::from_secs(10),
        );
        let second = start_slot(
            &mut machine,
            spectator::SpectateSlot::Stream,
            &zone,
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::from_secs(11),
        );
        assert_eq!(
            testing::count_session_outputs(&first, |output| matches!(
                output,
                spectator::state_machine::SpectateOutput::PublishLease { .. }
            )),
            1
        );
        assert_eq!(
            testing::count_session_outputs(&first, |output| matches!(
                output,
                spectator::state_machine::SpectateOutput::Subscribe(_)
            )),
            1
        );
        assert_eq!(
            testing::count_session_outputs(&second, |output| matches!(
                output,
                spectator::state_machine::SpectateOutput::PublishLease { .. }
            )),
            0
        );
        assert_eq!(
            testing::count_session_outputs(&second, |output| matches!(
                output,
                spectator::state_machine::SpectateOutput::Subscribe(_)
            )),
            0
        );
    }

    #[test]
    fn replacing_a_slot_zone_leaves_the_old_zone_and_keeps_the_other_slot() {
        let mut machine = testing::machine();
        machine.me = Some(identity::PeerId::from("me"));
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
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::from_secs(11),
        );
        let outputs = start_slot(
            &mut machine,
            spectator::SpectateSlot::Preview,
            &identity::ZoneName::from("other-zone"),
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::from_secs(12),
        );
        assert!(
            testing::session_outputs(&outputs)
                .iter()
                .any(|output| matches!(
                    output,
                    spectator::state_machine::SpectateOutput::UnwatchZone(zone)
                        if *zone == identity::ZoneName::from("preview-zone")
                ))
        );
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
