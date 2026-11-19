use bevy::prelude::App;

use crate::spectator;
use crate::testing;

#[must_use]
pub fn has_spectated(app: &mut App, slot: spectator::SpectateSlot) -> bool {
    let mut query = app
        .world_mut()
        .query::<&spectator::Spectated<testing::KeyValueMap>>();
    let world = app.world();
    query.iter(world).any(|spectated| spectated.slot == slot)
}

#[cfg(test)]
mod tests {
    use crate::spectator;
    use crate::testing;

    use super::has_spectated;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        assert!(!has_spectated(&mut app, spectator::SpectateSlot::Preview));
        app.world_mut()
            .spawn(testing::spectated_state(testing::KeyValueMap::default()));
        assert!(has_spectated(&mut app, spectator::SpectateSlot::Preview));
        assert!(!has_spectated(&mut app, spectator::SpectateSlot::Stream));
    }
}
