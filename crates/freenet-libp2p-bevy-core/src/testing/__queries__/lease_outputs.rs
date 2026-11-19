use crate::spectator;
use crate::time;

#[must_use]
pub fn lease_outputs(
    outputs: &[spectator::state_machine::SpectateOutput<u8, Vec<u8>>],
) -> Vec<time::UnixTime> {
    outputs
        .iter()
        .filter_map(|output| match output {
            spectator::state_machine::SpectateOutput::PublishLease { until, .. } => Some(*until),
            _ => None,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::spectator;
    use crate::time;

    use super::lease_outputs;

    #[test]
    fn test_usage() {
        let until = time::UnixTime::from_secs(5);
        let outputs = [
            spectator::state_machine::SpectateOutput::PublishLease {
                zone: identity::ZoneName::from("zone"),
                spectator: identity::PeerId::from("me"),
                until,
            },
            spectator::state_machine::SpectateOutput::RepairFinished,
        ];
        assert_eq!(lease_outputs(&outputs), vec![until]);
    }
}
