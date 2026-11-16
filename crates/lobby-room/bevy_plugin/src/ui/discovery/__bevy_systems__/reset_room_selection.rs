use crate::ui;
use bevy::prelude::*;

use ui::discovery::SelectedRoom;

pub fn reset_room_selection(mut selection: ResMut<SelectedRoom>) {
    *selection = SelectedRoom::default();
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::reset_room_selection;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.insert_resource(ui::discovery::SelectedRoom {
            room: Some(core_discovery::RoomName::from("alpha")),
            is_locked: true,
        });
        app.add_systems(Update, reset_room_selection);
        app.update();
        let selection = app.world().resource::<ui::discovery::SelectedRoom>();
        assert_eq!(selection.room, None);
        assert!(!selection.is_locked);
    }
}
