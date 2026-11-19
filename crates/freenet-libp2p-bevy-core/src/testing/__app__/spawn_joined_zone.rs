use bevy::prelude::{App, Entity};

use crate::identity;
use crate::p2p;
use crate::peers;
use crate::replica_sync;

#[must_use]
pub fn spawn_joined_zone(
    app: &mut App,
    zone: &identity::ZoneName,
    members: Vec<identity::PeerId>,
    prefix: p2p::HistoryKeyPrefix,
    liveness: replica_sync::state_machine::ZoneLiveness,
) -> Entity {
    let zone_entity = app.world_mut().spawn((zone.clone(), prefix, liveness)).id();
    app.world_mut().spawn((
        peers::Peer::from(identity::PeerId::from("me")),
        peers::Me,
        peers::InZone(zone_entity),
    ));
    for member in members {
        app.world_mut().spawn((
            peers::Peer::from(member),
            peers::InZone(zone_entity),
            peers::Connected,
        ));
    }
    zone_entity
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::p2p;
    use crate::replica_sync;
    use crate::testing;

    use super::spawn_joined_zone;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        let zone_entity = spawn_joined_zone(
            &mut app,
            &identity::ZoneName::from("zone"),
            vec![identity::PeerId::from("member")],
            p2p::HistoryKeyPrefix::from("prefix"),
            replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed: true,
                has_other_history_holders: false,
            },
        );
        assert!(app.world().get::<identity::ZoneName>(zone_entity).is_some());
    }
}
