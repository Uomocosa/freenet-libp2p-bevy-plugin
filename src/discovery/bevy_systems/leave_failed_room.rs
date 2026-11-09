use bevy::prelude::*;

use crate::discovery;
use crate::room_sync;

pub fn leave_failed_room(
    failures: Option<MessageReader<room_sync::JoinFailure>>,
    mut inputs: MessageWriter<discovery::state_machine::Input>,
) {
    let Some(mut failures) = failures else {
        return;
    };
    if failures.read().next().is_some() {
        inputs.write(discovery::state_machine::Input::Command(
            discovery::Command::Leave,
        ));
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::leave_failed_room;
    use crate::discovery;
    use crate::identity;
    use crate::room_sync;

    fn app_with_system() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<room_sync::JoinFailure>();
        app.add_message::<discovery::state_machine::Input>();
        app.add_systems(Update, leave_failed_room);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with_system();
        app.world_mut()
            .resource_mut::<Messages<room_sync::JoinFailure>>()
            .write(room_sync::JoinFailure::RoomGone(identity::RoomName::from(
                "room-a",
            )));
        app.update();
        let inputs: Vec<discovery::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(
            inputs,
            vec![discovery::state_machine::Input::Command(
                discovery::Command::Leave
            )]
        );
        app.update();
        let count = app
            .world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .drain()
            .count();
        assert_eq!(count, 0);
    }

    #[test]
    fn two_failures_in_one_frame_send_a_single_leave() {
        let mut app = app_with_system();
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
        let inputs: Vec<discovery::state_machine::Input> = app
            .world_mut()
            .resource_mut::<Messages<discovery::state_machine::Input>>()
            .drain()
            .collect();
        assert_eq!(
            inputs,
            vec![discovery::state_machine::Input::Command(
                discovery::Command::Leave
            )]
        );
    }
}
