use crate::identity;
use crate::spectator;
use crate::time;

pub fn update_sessions_on_zone<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    zone: &identity::ZoneName,
    input: spectator::state_machine::SpectateInput<Op, Vec<u8>>,
    now: time::UnixTime,
) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    let slots: Vec<spectator::SpectateSlot> = machine
        .sessions
        .iter()
        .filter(|(_, session)| session.zone.as_ref() == Some(zone))
        .map(|(slot, _)| *slot)
        .collect();
    let Some((last, leading)) = slots.split_last() else {
        return Vec::new();
    };
    let mut outputs = Vec::new();
    for slot in leading {
        outputs.extend(machine.update_session(*slot, input.clone(), now));
    }
    outputs.extend(machine.update_session(*last, input, now));
    outputs
}

#[cfg(test)]
mod tests {
    use super::update_sessions_on_zone;
    use crate::identity;
    use crate::spectator;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut machine = spectator::Machine::<u8>::new(identity::GameId::from("game"));
        assert_eq!(
            update_sessions_on_zone(
                &mut machine,
                &identity::ZoneName::from("zone"),
                spectator::state_machine::SpectateInput::Tick,
                time::UnixTime::EPOCH,
            ),
            Vec::new()
        );
    }
}
