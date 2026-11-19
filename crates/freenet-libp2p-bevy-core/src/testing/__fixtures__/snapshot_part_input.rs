use crate::identity;
use crate::replica_sync;

#[must_use]
pub fn snapshot_part_input<Op>(part: u32, total: u32) -> replica_sync::state_machine::Input<Op> {
    replica_sync::state_machine::Input::SnapshotPartReceivedFromZoneMember {
        zone: identity::ZoneName::from("r"),
        part,
        total,
        data: vec![7],
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::replica_sync;

    use super::snapshot_part_input;

    #[test]
    fn test_usage() {
        let input = snapshot_part_input::<u8>(1, 3);
        let replica_sync::state_machine::Input::SnapshotPartReceivedFromZoneMember {
            zone,
            part,
            total,
            data,
        } = input
        else {
            panic!("expected a snapshot part input");
        };
        assert_eq!(zone, identity::ZoneName::from("r"));
        assert_eq!((part, total), (1, 3));
        assert_eq!(data, vec![7]);
    }
}
