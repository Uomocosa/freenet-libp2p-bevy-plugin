use bevy::prelude::App;

use crate::discovery;
use crate::ui;

pub fn stream_room(app: &mut App, room: Option<&str>) {
    **app
        .world_mut()
        .resource_mut::<ui::discovery::StreamedRoom>() = room.map(discovery::RoomName::from);
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::testing;
    use crate::ui;

    use super::stream_room;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        app.insert_resource(ui::discovery::StreamedRoom::default());
        stream_room(&mut app, Some("alpha"));
        assert_eq!(
            **app.world().resource::<ui::discovery::StreamedRoom>(),
            Some(discovery::RoomName::from("alpha"))
        );
    }
}
