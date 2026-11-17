use crate::spectator;
use crate::time;

pub fn update_session<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    slot: spectator::SpectateSlot,
    input: spectator::state_machine::SpectateInput<Op, Vec<u8>>,
    now: time::UnixTime,
) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    let lease_interval = machine.in_zone.lease_interval();
    let Some(session) = machine.sessions.get_mut(&slot) else {
        return Vec::new();
    };
    let session_outputs = session.update(input, now, lease_interval);
    machine.forward_outputs(slot, session_outputs, false)
}

#[cfg(test)]
mod tests {
    use super::update_session;
    use crate::identity;
    use crate::spectator;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut machine = spectator::Machine::<u8>::new(identity::GameId::from("game"));
        assert_eq!(
            update_session(
                &mut machine,
                spectator::SpectateSlot::Preview,
                spectator::state_machine::SpectateInput::Tick,
                time::UnixTime::EPOCH,
            ),
            Vec::new()
        );
    }
}
