use bevy::prelude::App;

use crate::identity;
use crate::p2p;
use crate::spectator;

pub fn spectate(
    app: &mut App,
    slot: spectator::SpectateSlot,
    zone: identity::ZoneName,
    history_prefix: p2p::HistoryKeyPrefix,
) {
    app.world_mut()
        .write_message(spectator::SpectateCommand::Start {
            slot,
            zone,
            history_prefix,
        });
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::p2p;
    use crate::spectator;
    use crate::testing;

    use super::spectate;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        app.add_message::<spectator::SpectateCommand>();
        spectate(
            &mut app,
            spectator::SpectateSlot::Preview,
            identity::ZoneName::from("zone"),
            p2p::HistoryKeyPrefix::from("prefix"),
        );
        assert_eq!(
            app.world()
                .resource::<bevy::prelude::Messages<spectator::SpectateCommand>>()
                .len(),
            1
        );
    }
}
