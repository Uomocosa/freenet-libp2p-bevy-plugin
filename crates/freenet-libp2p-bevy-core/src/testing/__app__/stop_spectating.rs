use bevy::prelude::App;

use crate::spectator;

pub fn stop_spectating(app: &mut App, slot: spectator::SpectateSlot) {
    app.world_mut()
        .write_message(spectator::SpectateCommand::Stop { slot });
}

#[cfg(test)]
mod tests {
    use bevy::prelude::Messages;

    use crate::spectator;
    use crate::testing;

    use super::stop_spectating;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        app.add_message::<spectator::SpectateCommand>();
        stop_spectating(&mut app, spectator::SpectateSlot::Preview);
        assert_eq!(
            app.world()
                .resource::<Messages<spectator::SpectateCommand>>()
                .len(),
            1
        );
    }
}
