use bevy::prelude::*;

use crate::discovery;
use crate::room_sync;

pub fn leave_failed_room(
    failures: Option<MessageReader<room_sync::JoinFailure>>,
    sender: Option<Res<discovery::CommandSender>>,
) {
    let Some(mut failures) = failures else {
        return;
    };
    let Some(sender) = sender.map(Res::into_inner) else {
        return;
    };
    if failures.read().next().is_some() {
        sender.send(discovery::Command::Leave).ok();
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::leave_failed_room;
    use crate::discovery;
    use crate::identity;
    use crate::room_sync;

    fn app_with_system() -> (
        App,
        tokio::sync::mpsc::UnboundedReceiver<discovery::Command>,
    ) {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<room_sync::JoinFailure>();
        app.insert_resource(discovery::CommandSender(tx));
        app.add_systems(Update, leave_failed_room);
        (app, rx)
    }

    #[test]
    fn test_usage() {
        let (mut app, mut rx) = app_with_system();
        app.world_mut()
            .resource_mut::<Messages<room_sync::JoinFailure>>()
            .write(room_sync::JoinFailure::RoomGone(identity::RoomName::from(
                "room-a",
            )));
        app.update();
        assert_eq!(rx.try_recv(), Ok(discovery::Command::Leave));
        app.update();
        assert!(rx.try_recv().is_err());
    }

    #[test]
    fn two_failures_in_one_frame_send_a_single_leave() {
        let (mut app, mut rx) = app_with_system();
        app.world_mut()
            .resource_mut::<Messages<room_sync::JoinFailure>>()
            .write(room_sync::JoinFailure::RoomGone(identity::RoomName::from(
                "room-a",
            )));
        app.world_mut()
            .resource_mut::<Messages<room_sync::JoinFailure>>()
            .write(room_sync::JoinFailure::HistoryUnavailable(
                identity::RoomName::from("room-b"),
            ));
        app.update();
        assert_eq!(rx.try_recv(), Ok(discovery::Command::Leave));
        assert!(rx.try_recv().is_err());
    }
}
