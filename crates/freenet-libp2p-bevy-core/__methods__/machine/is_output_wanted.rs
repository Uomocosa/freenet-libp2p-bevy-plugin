use crate::spectator;

pub fn is_output_wanted<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    slot: spectator::SpectateSlot,
    output: &spectator::state_machine::SpectateOutput<Op, Vec<u8>>,
    is_stopping: bool,
) -> bool {
    match output {
        spectator::state_machine::SpectateOutput::PublishLease { zone, .. } => {
            if is_stopping {
                !machine.does_any_slot_watch(zone)
            } else {
                machine.is_lease_owner(slot, zone)
            }
        }
        spectator::state_machine::SpectateOutput::Subscribe(topic) => {
            machine.note_topic_subscription(topic)
        }
        spectator::state_machine::SpectateOutput::Unsubscribe(topic) => {
            machine.release_topic_subscription(topic)
        }
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::is_output_wanted;
    use crate::identity;
    use crate::spectator;

    #[test]
    fn test_usage() {
        let mut machine = spectator::Machine::<u8>::new(identity::GameId::from("game"));
        assert!(is_output_wanted(
            &mut machine,
            spectator::SpectateSlot::Preview,
            &spectator::state_machine::SpectateOutput::WatchZone(identity::ZoneName::from("zone")),
            false,
        ));
    }
}
