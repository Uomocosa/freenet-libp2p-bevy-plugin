use crate::identity;
use crate::spectator;
use crate::time;

pub fn update_local_peer<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    peer: Option<identity::PeerId>,
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
            spectator::state_machine::SpectateInput::LocalPeer(peer.clone()),
            now,
        ));
    }
    outputs.extend(machine.update_session(
        *last,
        spectator::state_machine::SpectateInput::LocalPeer(peer),
        now,
    ));
    outputs
}

#[cfg(test)]
mod tests {
    use super::update_local_peer;
    use crate::identity;
    use crate::spectator;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut machine = spectator::Machine::<u8>::new(identity::GameId::from("game"));
        for slot in [
            spectator::SpectateSlot::Preview,
            spectator::SpectateSlot::Stream,
        ] {
            machine.sessions.insert(
                slot,
                spectator::state_machine::SpectatorState::new(identity::GameId::from("game")),
            );
        }
        assert_eq!(
            update_local_peer(
                &mut machine,
                Some(identity::PeerId::from("me")),
                time::UnixTime::EPOCH
            ),
            Vec::new()
        );
        assert!(
            machine
                .sessions
                .values()
                .all(|session| session.me == Some(identity::PeerId::from("me")))
        );
    }
}
