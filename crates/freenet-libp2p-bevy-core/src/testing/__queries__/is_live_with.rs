use bevy::prelude::App;

use crate::spectator;
use crate::testing;

#[must_use]
pub fn is_live_with(app: &mut App, slot: spectator::SpectateSlot, key: u8, value: u32) -> bool {
    let mut query = app
        .world_mut()
        .query::<&spectator::Spectated<testing::KeyValueMap>>();
    let world = app.world();
    query.iter(world).any(|spectated| {
        spectated.slot == slot
            && spectated.freshness == spectator::SpectateFreshness::Live
            && spectated.state.entries.get(&key) == Some(&value)
    })
}

#[cfg(test)]
mod tests {
    use crate::spectator;
    use crate::testing;

    use super::is_live_with;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        let mut state = testing::KeyValueMap::default();
        state.entries.insert(1, 10);
        let mut spectated = testing::spectated_state(state);
        spectated.freshness = spectator::SpectateFreshness::Live;
        app.world_mut().spawn(spectated);
        assert!(is_live_with(
            &mut app,
            spectator::SpectateSlot::Preview,
            1,
            10
        ));
        assert!(!is_live_with(
            &mut app,
            spectator::SpectateSlot::Stream,
            1,
            10
        ));
    }
}
