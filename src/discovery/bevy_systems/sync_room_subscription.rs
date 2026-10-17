use bevy::prelude::*;

use crate::discovery;
use crate::p2p;

pub fn sync_room_subscription(
    mut events: MessageReader<discovery::Event>,
    mut pending: ResMut<p2p::PendingNetCommands>,
) {
    for event in events.read() {
        match event {
            discovery::Event::Joined(room) => {
                pending.push(p2p::NetCommand::SubscribeRoom { room: room.clone() });
            }
            discovery::Event::Left(room) => {
                pending.push(p2p::NetCommand::UnsubscribeRoom { room: room.clone() });
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::sync_room_subscription;
    use crate::discovery;
    use crate::identity;
    use crate::p2p;

    fn event_app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<discovery::Event>();
        app.init_resource::<p2p::PendingNetCommands>();
        app.add_systems(Update, sync_room_subscription);
        app
    }

    fn send(app: &mut App, event: discovery::Event) {
        app.world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(event);
        app.update();
    }

    #[test]
    fn test_usage() {
        let mut app = event_app();
        let room = identity::RoomName::from("room-a");
        send(&mut app, discovery::Event::Joined(room.clone()));
        assert_eq!(
            app.world().resource::<p2p::PendingNetCommands>().as_slice(),
            [p2p::NetCommand::SubscribeRoom { room: room.clone() }]
        );
        send(&mut app, discovery::Event::Left(room.clone()));
        assert_eq!(
            app.world().resource::<p2p::PendingNetCommands>().as_slice(),
            [
                p2p::NetCommand::SubscribeRoom { room: room.clone() },
                p2p::NetCommand::UnsubscribeRoom { room },
            ]
        );
    }

    #[test]
    fn unrelated_events_are_ignored() {
        let mut app = event_app();
        send(
            &mut app,
            discovery::Event::RoomReady(identity::RoomName::from("room-a")),
        );
        assert!(app.world().resource::<p2p::PendingNetCommands>().is_empty());
    }
}
