use std::time::Duration;

use bevy::prelude::{App, MinimalPlugins, Plugin};
use bevy::time::TimeUpdateStrategy;
use derive_more::{Deref, DerefMut};

#[derive(Deref, DerefMut)]
pub struct TestApp(App);

impl Default for TestApp {
    fn default() -> Self {
        Self::new()
    }
}

impl TestApp {
    #[must_use]
    pub fn new() -> Self {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                16,
            )));
        Self(app)
    }

    #[must_use]
    pub fn with_frame(mut self, frame: Duration) -> Self {
        self.insert_resource(TimeUpdateStrategy::ManualDuration(frame));
        self
    }

    #[must_use]
    pub fn with_plugin(mut self, plugin: impl Plugin) -> Self {
        self.add_plugins(plugin);
        self
    }

    pub fn tick(&mut self, frames: u32) {
        for _ in 0..frames {
            self.update();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::input::InputPlugin;
    use bevy::prelude::Time;

    use super::TestApp;

    #[test]
    fn test_usage() {
        let mut app = TestApp::new()
            .with_frame(Duration::from_millis(50))
            .with_plugin(InputPlugin);
        app.tick(2);
        assert_eq!(
            app.world().resource::<Time>().delta(),
            Duration::from_millis(50)
        );
    }
}
