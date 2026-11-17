use crate::identity;
use crate::replica_sync;
use crate::spectator;
use crate::time;

pub fn route_slice_received<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    zone: &identity::ZoneName,
    index: replica_sync::ReplicaSliceIndex,
    snapshot: Vec<u8>,
    now: time::UnixTime,
) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    machine.update_sessions_on_zone(
        zone,
        spectator::state_machine::SpectateInput::SliceReceived {
            zone: zone.clone(),
            index,
            snapshot,
        },
        now,
    )
}

#[cfg(test)]
mod tests {
    use super::route_slice_received;
    use crate::identity;
    use crate::replica_sync;
    use crate::spectator;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut machine = spectator::Machine::<u8>::new(identity::GameId::from("game"));
        assert_eq!(
            route_slice_received(
                &mut machine,
                &identity::ZoneName::from("zone"),
                replica_sync::ReplicaSliceIndex(0),
                Vec::new(),
                time::UnixTime::EPOCH,
            ),
            Vec::new()
        );
    }
}
