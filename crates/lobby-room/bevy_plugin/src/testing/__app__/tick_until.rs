use bevy::prelude::App;

pub fn tick_until(
    app: &mut App,
    rounds: usize,
    mut is_ready: impl FnMut(&mut App) -> bool,
) -> bool {
    for _ in 0..rounds {
        app.update();
        if is_ready(app) {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::prelude::Time;

    use crate::testing;

    use super::tick_until;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        assert!(!tick_until(&mut app, 1, |_app| false));
        assert!(tick_until(&mut app, 400, |app| {
            app.world().resource::<Time>().elapsed() > Duration::from_millis(30)
        }));
    }
}
