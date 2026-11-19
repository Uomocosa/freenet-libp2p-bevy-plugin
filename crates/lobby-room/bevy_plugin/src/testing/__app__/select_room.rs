use bevy::prelude::App;

use crate::discovery;
use crate::ui;

pub fn select_room(app: &mut App, room: Option<&str>) {
    **app
        .world_mut()
        .resource_mut::<ui::discovery::SelectedRoom>() = room.map(discovery::RoomName::from);
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use crate::testing;
    use crate::ui;

    use super::select_room;

    #[test]
    fn test_usage() {
        let mut app = testing::TestApp::new();
        app.insert_resource(ui::discovery::SelectedRoom::default());
        select_room(&mut app, Some("alpha"));
        assert_eq!(
            **app.world().resource::<ui::discovery::SelectedRoom>(),
            Some(discovery::RoomName::from("alpha"))
        );
    }
}
