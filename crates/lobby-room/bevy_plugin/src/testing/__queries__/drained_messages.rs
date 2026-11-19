use bevy::prelude::{App, Message, Messages};

pub fn drained_messages<M: Message>(app: &mut App) -> Vec<M> {
    app.world_mut()
        .resource_mut::<Messages<M>>()
        .drain()
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::spectator;
    use crate::testing;

    use super::drained_messages;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        app.add_message::<spectator::SpectateCommand>();
        app.world_mut()
            .write_message(spectator::SpectateCommand::Stop {
                slot: spectator::SpectateSlot::Preview,
            });
        assert_eq!(
            drained_messages::<spectator::SpectateCommand>(&mut app),
            vec![spectator::SpectateCommand::Stop {
                slot: spectator::SpectateSlot::Preview,
            }]
        );
        assert_eq!(
            drained_messages::<spectator::SpectateCommand>(&mut app),
            Vec::new()
        );
    }
}
