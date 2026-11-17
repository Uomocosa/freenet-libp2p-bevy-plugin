use crate::spectator;

pub fn forward_outputs<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    slot: spectator::SpectateSlot,
    outputs: Vec<spectator::state_machine::SpectateOutput<Op, Vec<u8>>>,
    is_stopping: bool,
) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    let mut forwarded = Vec::new();
    for output in outputs {
        if machine.is_output_wanted(slot, &output, is_stopping) {
            forwarded.push(spectator::MachineOutput::Session { slot, output });
        }
    }
    forwarded
}

#[cfg(test)]
mod tests {
    use super::forward_outputs;
    use crate::identity;
    use crate::spectator;

    fn machine() -> spectator::Machine<u8> {
        spectator::Machine::new(identity::GameId::from("game"))
    }

    #[test]
    fn test_usage() {
        let mut machine = machine();
        let outputs = forward_outputs(
            &mut machine,
            spectator::SpectateSlot::Preview,
            vec![spectator::state_machine::SpectateOutput::WatchZone(
                identity::ZoneName::from("zone"),
            )],
            false,
        );
        assert_eq!(outputs.len(), 1);
    }

    #[test]
    fn a_second_subscription_to_one_topic_is_dropped_until_the_last_unsubscribe() {
        let mut machine = machine();
        let topic = identity::Topic::from("game/zone");
        let subscribe =
            || spectator::state_machine::SpectateOutput::<u8, Vec<u8>>::Subscribe(topic.clone());
        let first = forward_outputs(
            &mut machine,
            spectator::SpectateSlot::Preview,
            vec![subscribe()],
            false,
        );
        let second = forward_outputs(
            &mut machine,
            spectator::SpectateSlot::Stream,
            vec![subscribe()],
            false,
        );
        assert_eq!(first.len(), 1);
        assert_eq!(second, Vec::new());
        let first_stop = forward_outputs(
            &mut machine,
            spectator::SpectateSlot::Preview,
            vec![spectator::state_machine::SpectateOutput::Unsubscribe(
                topic.clone(),
            )],
            true,
        );
        let last_stop = forward_outputs(
            &mut machine,
            spectator::SpectateSlot::Stream,
            vec![spectator::state_machine::SpectateOutput::Unsubscribe(
                topic.clone(),
            )],
            true,
        );
        assert_eq!(first_stop, Vec::new());
        assert_eq!(last_stop.len(), 1);
    }
}
