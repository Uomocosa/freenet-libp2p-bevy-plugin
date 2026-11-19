use bevy::prelude::App;

use crate::spectator;

#[must_use]
pub fn spectator_count(app: &App) -> usize {
    app.world()
        .get_resource::<spectator::SpectatorCount>()
        .map_or(0, |count| **count)
}

#[cfg(test)]
mod tests {
    use crate::spectator;
    use crate::testing;

    use super::spectator_count;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        assert_eq!(spectator_count(&app), 0);
        app.insert_resource(spectator::SpectatorCount(3));
        assert_eq!(spectator_count(&app), 3);
    }
}
