use bevy::prelude::*;

use crate::discovery;

pub fn notify_outputs_to_events(
    mut outputs: MessageReader<discovery::state_machine::Output>,
    mut events: MessageWriter<discovery::Event>,
) {
    for output in outputs.read() {
        if let discovery::state_machine::Output::Notify(event) = output {
            events.write(event.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::notify_outputs_to_events;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::state_machine::Output>();
        app.add_message::<discovery::Event>();
        app.add_systems(Update, notify_outputs_to_events);
        app.world_mut()
            .resource_mut::<Messages<discovery::state_machine::Output>>()
            .write(discovery::state_machine::Output::Notify(
                discovery::Event::LobbyUpdated,
            ));
        app.update();
        let mut messages = app.world_mut().resource_mut::<Messages<discovery::Event>>();
        assert_eq!(messages.drain().count(), 1);
    }

    #[test]
    fn libp2p_outputs_write_no_event() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::state_machine::Output>();
        app.add_message::<discovery::Event>();
        app.add_systems(Update, notify_outputs_to_events);
        app.world_mut()
            .resource_mut::<Messages<discovery::state_machine::Output>>()
            .write(discovery::state_machine::Output::Libp2pCommand(
                p2p::Libp2pCommand::MeasureHistoryStore,
            ));
        app.update();
        let mut messages = app.world_mut().resource_mut::<Messages<discovery::Event>>();
        assert_eq!(messages.drain().count(), 0);
        let _ = identity::RoomName::from("unused");
    }
}
