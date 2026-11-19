use crate::identity;
use crate::replica_sync;

pub fn topic_message_to_input<Replica: replica_sync::traits::ReplicatedState>(
    message: replica_sync::ReplicaSyncTopicMessage<Replica::Op>,
    from: &identity::PeerId,
    clock: &mut replica_sync::LamportClock,
    replica: &Replica,
) -> Option<replica_sync::state_machine::Input<Replica::Op>> {
    match message {
        replica_sync::ReplicaSyncTopicMessage::ApplyThisOp { zone, meta, op } => {
            clock.observe(meta.lamport);
            Some(replica_sync::state_machine::Input::Remote { zone, meta, op })
        }
        replica_sync::ReplicaSyncTopicMessage::CompareSliceContentHashes { hashes, .. } => {
            tracing::debug!(target: "replica_sync", "anti-entropy compare from={from} remote_slices={}", hashes.len());
            Some(
                replica_sync::state_machine::Input::RemoteSliceContentHashes {
                    from: from.clone(),
                    local: replica.slice_content_hashes(),
                    remote: hashes,
                },
            )
        }
        replica_sync::ReplicaSyncTopicMessage::SpectatorLease { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use super::topic_message_to_input;
    use crate::identity;
    use crate::replica_sync;
    use crate::testing;
    use crate::time;

    #[test]
    fn test_usage() {
        let mut clock = replica_sync::LamportClock::default();
        let replica = testing::FakeReplica::<u8, u8>::default();
        let message = replica_sync::ReplicaSyncTopicMessage::SpectatorLease {
            zone: identity::ZoneName::from("zone"),
            spectator: identity::PeerId::from("spectator"),
            until: time::UnixTime::from_secs(5),
        };
        assert_eq!(
            topic_message_to_input(
                message,
                &identity::PeerId::from("peer"),
                &mut clock,
                &replica
            ),
            None
        );
    }
}
