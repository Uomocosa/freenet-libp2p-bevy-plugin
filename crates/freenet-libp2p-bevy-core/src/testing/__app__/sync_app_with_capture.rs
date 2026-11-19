use bevy::prelude::Update;

use crate::identity;
use crate::replica_sync;
use crate::testing;

#[must_use]
pub fn sync_app_with_capture(game: identity::GameId) -> testing::TestApp {
    let mut app = testing::TestApp::new();
    app.insert_resource(replica_sync::SyncGame(game));
    app.init_resource::<replica_sync::bevy_systems::SyncState<u8>>();
    app.init_resource::<testing::CapturedOutputs<u8>>();
    app.add_systems(Update, testing::bevy_systems::capture_zone_inputs::<u8>);
    app
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::testing;

    use super::sync_app_with_capture;

    #[test]
    fn test_usage() {
        let app = sync_app_with_capture(identity::GameId::from("game"));
        assert!(
            app.world()
                .contains_resource::<testing::CapturedOutputs<u8>>()
        );
    }
}
