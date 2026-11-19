use bevy::prelude::App;

use crate::identity;
use crate::p2p;

#[must_use]
pub fn zone_message_count<P: p2p::traits::Message>(app: &App, zone: &identity::ZoneName) -> usize {
    app.world()
        .get_resource::<p2p::PendingEvents<P>>()
        .map_or(0, |events| {
            events
                .iter()
                .filter(
                    |event| matches!(event, p2p::Event::ZoneMessage { zone: seen, .. } if seen == zone),
                )
                .count()
        })
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::p2p;
    use crate::testing;

    use super::zone_message_count;

    #[test]
    fn test_usage() {
        let zone = identity::ZoneName::from("zone");
        let mut app = testing::TestApp::new();
        app.insert_resource(p2p::PendingEvents::<u8>(vec![
            p2p::Event::ZoneMessage {
                zone: zone.clone(),
                from: identity::PeerId::from("a"),
                payload: 7,
            },
            p2p::Event::ZoneMessage {
                zone: identity::ZoneName::from("other"),
                from: identity::PeerId::from("a"),
                payload: 7,
            },
        ]));
        assert_eq!(zone_message_count::<u8>(&app, &zone), 1);
    }
}
