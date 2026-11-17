use crate::identity;
use crate::spectator;
use crate::time;

pub fn update_players<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    players: Vec<identity::PeerId>,
    now: time::UnixTime,
) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    let slots: Vec<spectator::SpectateSlot> = machine.sessions.keys().copied().collect();
    let Some((last, leading)) = slots.split_last() else {
        return Vec::new();
    };
    let mut outputs = Vec::new();
    for slot in leading {
        outputs.extend(machine.update_session(
            *slot,
            spectator::state_machine::SpectateInput::Players(players.clone()),
            now,
        ));
    }
    outputs.extend(machine.update_session(
        *last,
        spectator::state_machine::SpectateInput::Players(players),
        now,
    ));
    outputs
}

#[cfg(test)]
mod tests {
    use super::update_players;
    use crate::identity;
    use crate::spectator;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut machine = spectator::Machine::<u8>::new(identity::GameId::from("game"));
        assert_eq!(
            update_players(
                &mut machine,
                vec![identity::PeerId::from("a")],
                time::UnixTime::EPOCH
            ),
            Vec::new()
        );
    }
}
