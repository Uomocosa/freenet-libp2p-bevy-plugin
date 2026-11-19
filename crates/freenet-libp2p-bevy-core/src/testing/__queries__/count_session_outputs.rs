use crate::spectator;
use crate::testing;

#[must_use]
pub fn count_session_outputs<Op>(
    outputs: &[spectator::MachineOutput<Op, Vec<u8>>],
    is_counted: impl Fn(&spectator::state_machine::SpectateOutput<Op, Vec<u8>>) -> bool,
) -> usize {
    testing::session_outputs(outputs)
        .into_iter()
        .filter(|output| is_counted(output))
        .count()
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::spectator;

    use super::count_session_outputs;

    #[test]
    fn test_usage() {
        let outputs = vec![spectator::MachineOutput::Session {
            slot: spectator::SpectateSlot::Preview,
            output: spectator::state_machine::SpectateOutput::<u8, Vec<u8>>::WatchZone(
                identity::ZoneName::from("zone"),
            ),
        }];
        assert_eq!(
            count_session_outputs(&outputs, |output| matches!(
                output,
                spectator::state_machine::SpectateOutput::WatchZone(_)
            )),
            1
        );
    }
}
