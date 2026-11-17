use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::StreamedRoom;

pub fn drop_stale_streamed_room(
    rows: Option<Res<core_discovery::LobbyRows>>,
    mut streamed: ResMut<StreamedRoom>,
) {
    let Some(room) = (**streamed).clone() else {
        return;
    };
    let rows = rows.map(Res::into_inner);
    let is_present = rows.is_some_and(|rows| rows.iter().any(|row| row.name == room));
    if !is_present {
        **streamed = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::drop_stale_streamed_room;

    fn row(name: &str) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants: 0,
            room_size: core_discovery::RoomSize::DEFAULT,
            occupancy: core_discovery::RoomOccupancy::Empty,
            friends: 0,
        }
    }

    fn app_with(room: Option<&str>, rows: Vec<core_discovery::LobbyRow>) -> App {
        let mut app = App::new();
        app.insert_resource(core_discovery::LobbyRows(rows));
        app.insert_resource(ui::discovery::StreamedRoom(
            room.map(core_discovery::RoomName::from),
        ));
        app.add_systems(Update, drop_stale_streamed_room);
        app
    }

    fn streamed(app: &App) -> Option<core_discovery::RoomName> {
        (**app.world().resource::<ui::discovery::StreamedRoom>()).clone()
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(Some("alpha"), vec![row("alpha")]);
        app.update();
        assert_eq!(
            streamed(&app),
            Some(core_discovery::RoomName::from("alpha"))
        );
    }

    #[test]
    fn a_streamed_room_that_vanishes_is_cleared() {
        let mut app = app_with(Some("alpha"), vec![row("beta")]);
        app.update();
        assert_eq!(streamed(&app), None);
    }
}
