use bevy::prelude::App;

use crate::identity;
use crate::p2p;
use crate::peers;
use crate::replica_sync;
use crate::testing;

pub fn join_player(app: &mut App, zone: &identity::ZoneName, prefix: &p2p::HistoryKeyPrefix) {
    let zone_entity = app
        .world_mut()
        .spawn((
            zone.clone(),
            prefix.clone(),
            replica_sync::state_machine::ZoneLiveness {
                is_zone_still_listed: true,
                has_other_history_holders: false,
            },
        ))
        .id();
    let self_id =
        testing::ready_peer(app).map_or_else(|| identity::PeerId::from("me"), |peer| peer.id);
    app.world_mut().spawn((
        peers::Peer::from(self_id),
        peers::Me,
        peers::InZone(zone_entity),
    ));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::With;

    use crate::identity;
    use crate::p2p;
    use crate::peers;
    use crate::testing;

    use super::join_player;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        join_player(
            &mut app,
            &identity::ZoneName::from("zone"),
            &p2p::HistoryKeyPrefix::from("prefix"),
        );
        assert_eq!(
            testing::count_matching::<(), With<peers::Me>>(app.world_mut()),
            1
        );
        assert_eq!(
            testing::count_matching::<(), With<identity::ZoneName>>(app.world_mut()),
            1
        );
    }
}
