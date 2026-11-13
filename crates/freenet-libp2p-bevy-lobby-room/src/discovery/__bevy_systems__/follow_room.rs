use bevy::prelude::*;

use crate::discovery;
use crate::p2p;

pub fn follow_room(
    mut events: MessageReader<discovery::Event>,
    mut commands: MessageWriter<p2p::Libp2pCommand>,
) {
    for event in events.read() {
        match event {
            discovery::Event::Joined(room) => {
                commands.write(p2p::Libp2pCommand::SubscribeRoom { room: room.clone() });
            }
            discovery::Event::Left(room) => {
                commands.write(p2p::Libp2pCommand::UnsubscribeRoom { room: room.clone() });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::follow_room;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;

    fn event_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::Event>();
        app.add_message::<p2p::Libp2pCommand>();
        app.add_systems(Update, follow_room);
        app
    }

    fn send(app: &mut App, event: discovery::Event) {
        app.world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(event);
        app.update();
    }

    fn drain_commands(app: &mut App) -> Vec<p2p::Libp2pCommand> {
        app.world_mut()
            .resource_mut::<Messages<p2p::Libp2pCommand>>()
            .drain()
            .collect()
    }

    #[test]
    fn test_usage() {
        let mut app = event_app();
        let room = identity::RoomName::from("room-a");
        send(&mut app, discovery::Event::Joined(room.clone()));
        assert_eq!(
            drain_commands(&mut app),
            [p2p::Libp2pCommand::SubscribeRoom { room: room.clone() }]
        );
        send(&mut app, discovery::Event::Left(room.clone()));
        assert_eq!(
            drain_commands(&mut app),
            [p2p::Libp2pCommand::UnsubscribeRoom { room }]
        );
    }

    #[test]
    fn unrelated_events_are_ignored() {
        let mut app = event_app();
        send(
            &mut app,
            discovery::Event::RoomReady(identity::RoomName::from("room-a")),
        );
        assert_eq!(drain_commands(&mut app), Vec::new());
    }
}
