use bevy::prelude::App;

pub fn tick(app: &mut App, frames: u32) {
    for _ in 0..frames {
        app.update();
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::Time;

    use crate::testing;

    use super::tick;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        tick(&mut app, 2);
        assert!(app.world().resource::<Time>().elapsed() > Duration::ZERO);
    }
}
