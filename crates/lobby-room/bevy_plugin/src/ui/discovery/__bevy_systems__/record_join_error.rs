use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::JoinError;

pub fn record_join_error(
    mut events: MessageReader<core_discovery::Event>,
    mut join_error: ResMut<JoinError>,
) {
    for event in events.read() {
        match event {
            core_discovery::Event::RoomGone(room) => {
                **join_error = Some(format!(
                    "Room {} no longer exists (all players left)",
                    room.as_str()
                ));
            }
            core_discovery::Event::HistoryUnavailable(_room) => {
                **join_error = Some(String::from("Couldn't get room history"));
            }
            core_discovery::Event::Joined(_) => {
                **join_error = None;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::record_join_error;
    use freenet_libp2p_bevy_core::identity;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<core_discovery::Event>();
        app.init_resource::<ui::discovery::JoinError>();
        app.add_systems(Update, record_join_error);
        app.world_mut()
            .resource_mut::<Messages<core_discovery::Event>>()
            .write(core_discovery::Event::RoomGone(identity::RoomName::from(
                "alpha",
            )));
        app.update();
        assert_eq!(
            **app.world().resource::<ui::discovery::JoinError>(),
            Some(String::from(
                "Room alpha no longer exists (all players left)"
            ))
        );
    }
}
