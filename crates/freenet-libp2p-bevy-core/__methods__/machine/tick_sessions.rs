use crate::spectator;
use crate::time;

pub fn tick_sessions<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    now: time::UnixTime,
) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    let slots: Vec<spectator::SpectateSlot> = machine.sessions.keys().copied().collect();
    let mut outputs = Vec::new();
    for slot in slots {
        outputs.extend(machine.update_session(
            slot,
            spectator::state_machine::SpectateInput::Tick,
            now,
        ));
    }
    outputs
}

#[cfg(test)]
mod tests {
    use super::tick_sessions;
    use crate::identity;
    use crate::spectator;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut machine = spectator::Machine::<u8>::new(identity::GameId::from("game"));
        assert_eq!(
            tick_sessions(&mut machine, time::UnixTime::EPOCH),
            Vec::new()
        );
    }
}
