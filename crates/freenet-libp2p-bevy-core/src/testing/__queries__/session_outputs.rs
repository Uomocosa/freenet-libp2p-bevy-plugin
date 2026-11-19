use crate::spectator;

#[must_use]
pub fn session_outputs<Op>(
    outputs: &[spectator::MachineOutput<Op, Vec<u8>>],
) -> Vec<&spectator::state_machine::SpectateOutput<Op, Vec<u8>>> {
    outputs
        .iter()
        .filter_map(|output| match output {
            spectator::MachineOutput::Session { output, .. } => Some(output),
            spectator::MachineOutput::SpectatorCountChanged(_) => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::spectator;

    use super::session_outputs;

    #[test]
    fn test_usage() {
        let outputs = vec![
            spectator::MachineOutput::Session {
                slot: spectator::SpectateSlot::Preview,
                output: spectator::state_machine::SpectateOutput::<u8, Vec<u8>>::WatchZone(
                    identity::ZoneName::from("zone"),
                ),
            },
            spectator::MachineOutput::SpectatorCountChanged(3),
        ];
        assert_eq!(session_outputs(&outputs).len(), 1);
    }
}
