use bevy::prelude::*;

use crate::discovery;
use discovery::ui::JoinError;

pub fn record_join_error(
    mut events: MessageReader<discovery::Event>,
    mut join_error: ResMut<JoinError>,
) {
    for event in events.read() {
        match event {
            discovery::Event::RoomGone(room) => {
                **join_error = Some(format!(
                    "Room {} no longer exists (all players left)",
                    room.as_str()
                ));
            }
            discovery::Event::HistoryUnavailable(_room) => {
                **join_error = Some(String::from("Couldn't get room history"));
            }
            discovery::Event::Joined(_) => {
                **join_error = None;
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::record_join_error;
    use crate::discovery;
    use crate::identity;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<discovery::Event>();
        app.init_resource::<discovery::ui::JoinError>();
        app.add_systems(Update, record_join_error);
        app.world_mut()
            .resource_mut::<Messages<discovery::Event>>()
            .write(discovery::Event::RoomGone(identity::RoomName::from(
                "alpha",
            )));
        app.update();
        assert_eq!(
            **app.world().resource::<discovery::ui::JoinError>(),
            Some(String::from(
                "Room alpha no longer exists (all players left)"
            ))
        );
    }
}
