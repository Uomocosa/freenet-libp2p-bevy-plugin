use std::time::Duration;

use bevy::prelude::{Update, Window};
use bevy::window::PrimaryWindow;

use crate::testing;

#[must_use]
pub fn e2e_driver_app() -> testing::TestApp {
    let mut app = testing::TestApp::new()
        .with_frame(Duration::from_millis(100))
        .with_plugin(testing::E2EDriverPlugin::new(0));
    app.init_resource::<testing::Seen>();
    app.add_systems(Update, (testing::bevy_systems::record_events,));
    app.world_mut().spawn((Window::default(), PrimaryWindow));
    app
}

#[cfg(test)]
mod tests {
    use super::e2e_driver_app;

    #[test]
    fn test_usage() {
        let mut app = e2e_driver_app();
        app.tick(1);
        assert!(app.world().contains_resource::<crate::testing::Seen>());
    }
}
