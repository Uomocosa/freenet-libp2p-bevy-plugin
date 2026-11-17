use crate::identity;
use crate::replica_sync;
use crate::spectator;
use crate::time;

pub fn route_remote_slice_content_hashes<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    from: &identity::PeerId,
    zone: &identity::ZoneName,
    local: &replica_sync::ReplicaSliceContentHashes,
    remote: &replica_sync::ReplicaSliceContentHashes,
    now: time::UnixTime,
) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    machine.update_sessions_on_zone(
        zone,
        spectator::state_machine::SpectateInput::RemoteSliceContentHashes {
            from: from.clone(),
            zone: zone.clone(),
            local: local.clone(),
            remote: remote.clone(),
        },
        now,
    )
}

#[cfg(test)]
mod tests {
    use super::route_remote_slice_content_hashes;
    use crate::identity;
    use crate::spectator;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut machine = spectator::Machine::<u8>::new(identity::GameId::from("game"));
        assert_eq!(
            route_remote_slice_content_hashes(
                &mut machine,
                &identity::PeerId::from("holder"),
                &identity::ZoneName::from("zone"),
                &Vec::new(),
                &Vec::new(),
                time::UnixTime::EPOCH,
            ),
            Vec::new()
        );
    }
}
