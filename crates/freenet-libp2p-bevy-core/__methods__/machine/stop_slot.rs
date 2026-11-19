use crate::spectator;
use crate::time;

pub fn stop_slot<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    slot: spectator::SpectateSlot,
    now: time::UnixTime,
) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    let Some(mut session) = machine.sessions.remove(&slot) else {
        return Vec::new();
    };
    let session_outputs = session.stop_spectating(now);
    machine.forward_outputs(slot, session_outputs, true)
}

#[cfg(test)]
mod tests {
    use super::stop_slot;
    use crate::identity;
    use crate::methods::machine::start_slot;
    use crate::p2p;
    use crate::spectator;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut machine = testing::machine();
        machine.me = Some(identity::PeerId::from("me"));
        let zone = identity::ZoneName::from("zone");
        start_slot(
            &mut machine,
            spectator::SpectateSlot::Stream,
            &zone,
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::from_secs(10),
        );
        let outputs = stop_slot(
            &mut machine,
            spectator::SpectateSlot::Stream,
            time::UnixTime::from_secs(20),
        );
        assert_eq!(
            testing::count_session_outputs(&outputs, |output| matches!(
                output,
                spectator::state_machine::SpectateOutput::PublishLease { .. }
            )),
            1
        );
        assert!(machine.sessions.is_empty());
    }

    #[test]
    fn stopping_preview_leaves_stream_intact() {
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
            p2p::HistoryKeyPrefix::from("preview-prefix"),
            time::UnixTime::from_secs(11),
        );
        let outputs = stop_slot(
            &mut machine,
            spectator::SpectateSlot::Preview,
            time::UnixTime::from_secs(20),
        );
        assert_eq!(
            testing::count_session_outputs(&outputs, |output| matches!(
                output,
                spectator::state_machine::SpectateOutput::PublishLease { .. }
            )),
            1
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
            machine.zone_of(spectator::SpectateSlot::Stream),
            Some(&identity::ZoneName::from("stream-zone"))
        );
    }

    #[test]
    fn stopping_one_of_two_slots_on_one_zone_keeps_the_lease_and_the_subscription() {
        let mut machine = testing::machine();
        machine.me = Some(identity::PeerId::from("me"));
        let zone = identity::ZoneName::from("shared");
        start_slot(
            &mut machine,
            spectator::SpectateSlot::Preview,
            &zone,
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::from_secs(10),
        );
        start_slot(
            &mut machine,
            spectator::SpectateSlot::Stream,
            &zone,
            p2p::HistoryKeyPrefix::from("prefix"),
            time::UnixTime::from_secs(11),
        );
        let still_shared = stop_slot(
            &mut machine,
            spectator::SpectateSlot::Preview,
            time::UnixTime::from_secs(20),
        );
        assert_eq!(
            testing::count_session_outputs(&still_shared, |output| matches!(
                output,
                spectator::state_machine::SpectateOutput::PublishLease { .. }
            )),
            0
        );
        assert!(
            !testing::session_outputs(&still_shared)
                .iter()
                .any(|output| matches!(
                    output,
                    spectator::state_machine::SpectateOutput::Unsubscribe(_)
                ))
        );
        let last = stop_slot(
            &mut machine,
            spectator::SpectateSlot::Stream,
            time::UnixTime::from_secs(21),
        );
        assert_eq!(
            testing::count_session_outputs(&last, |output| matches!(
                output,
                spectator::state_machine::SpectateOutput::PublishLease { .. }
            )),
            1
        );
        assert!(
            testing::session_outputs(&last)
                .iter()
                .any(|output| matches!(
                    output,
                    spectator::state_machine::SpectateOutput::Unsubscribe(_)
                ))
        );
    }
}
