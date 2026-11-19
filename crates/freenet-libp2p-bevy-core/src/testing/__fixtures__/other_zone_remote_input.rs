use crate::identity;
use crate::replica_sync;
use crate::testing;

#[must_use]
pub fn other_zone_remote_input<Op>(op: Op) -> replica_sync::state_machine::Input<Op> {
    replica_sync::state_machine::Input::Remote {
        zone: identity::ZoneName::from("other"),
        meta: testing::meta(1, 1),
        op,
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::replica_sync;

    use super::other_zone_remote_input;

    #[test]
    fn test_usage() {
        let input = other_zone_remote_input(9u8);
        let replica_sync::state_machine::Input::Remote { zone, meta, op } = input else {
            panic!("expected a remote input");
        };
        assert_eq!(zone, identity::ZoneName::from("other"));
        assert_eq!(meta.lamport, 1);
        assert_eq!(op, 9u8);
    }
}
