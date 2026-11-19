use crate::replica_sync;
use crate::spectator;
use crate::time;

pub fn route_topic<Op: Clone>(
    machine: &mut spectator::Machine<Op>,
    message: replica_sync::ReplicaSyncTopicMessage<Op>,
    now: time::UnixTime,
) -> Vec<spectator::MachineOutput<Op, Vec<u8>>> {
    match message {
        replica_sync::ReplicaSyncTopicMessage::ApplyThisOp { zone, meta, op } => {
            let input = spectator::state_machine::SpectateInput::Topic(
                replica_sync::ReplicaSyncTopicMessage::ApplyThisOp {
                    zone: zone.clone(),
                    meta,
                    op,
                },
            );
            machine.update_sessions_on_zone(&zone, input, now)
        }
        replica_sync::ReplicaSyncTopicMessage::SpectatorLease {
            zone,
            spectator: peer,
            until,
        } => machine.update_in_zone(
            spectator::InZoneInput::Lease {
                zone,
                spectator: peer,
                until,
            },
            now,
        ),
        replica_sync::ReplicaSyncTopicMessage::CompareSliceContentHashes { .. } => Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::route_topic;
    use crate::identity;
    use crate::replica_sync;
    use crate::spectator;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut machine = testing::machine();
        assert_eq!(
            route_topic(
                &mut machine,
                replica_sync::ReplicaSyncTopicMessage::ApplyThisOp {
                    zone: identity::ZoneName::from("zone"),
                    meta: replica_sync::OpMeta {
                        lamport: 1,
                        author: 1,
                    },
                    op: 1_u8,
                },
                time::UnixTime::EPOCH,
            ),
            Vec::new()
        );
    }

    #[test]
    fn a_lease_counts_in_the_zone_only_when_it_is_our_zone() {
        let mut machine = testing::machine();
        machine.in_zone.my_zone = Some(identity::ZoneName::from("home"));
        let outputs = route_topic(
            &mut machine,
            replica_sync::ReplicaSyncTopicMessage::SpectatorLease {
                zone: identity::ZoneName::from("home"),
                spectator: identity::PeerId::from("watcher"),
                until: time::UnixTime::from_secs(10),
            },
            time::UnixTime::from_secs(1),
        );
        assert_eq!(
            outputs,
            vec![spectator::MachineOutput::SpectatorCountChanged(1)]
        );
    }
}
