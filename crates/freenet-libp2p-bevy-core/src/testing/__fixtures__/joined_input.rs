use crate::identity;
use crate::p2p;
use crate::replica_sync;

#[must_use]
pub fn joined_input<Op>(members: Vec<identity::PeerId>) -> replica_sync::state_machine::Input<Op> {
    replica_sync::state_machine::Input::Joined {
        zone: identity::ZoneName::from("r"),
        prefix: p2p::HistoryKeyPrefix::from("prefix"),
        members,
        liveness: replica_sync::state_machine::ZoneLiveness::default(),
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::replica_sync;

    use super::joined_input;

    #[test]
    fn test_usage() {
        let input = joined_input::<u8>(vec![identity::PeerId::from("a")]);
        let replica_sync::state_machine::Input::Joined {
            zone,
            members,
            liveness,
            ..
        } = input
        else {
            panic!("expected a joined input");
        };
        assert_eq!(zone, identity::ZoneName::from("r"));
        assert_eq!(members, vec![identity::PeerId::from("a")]);
        assert!(!liveness.is_zone_still_listed);
    }
}
