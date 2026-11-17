use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::SelectedRoom;

pub fn drop_stale_room_selection(
    rows: Option<Res<core_discovery::LobbyRows>>,
    mut selection: ResMut<SelectedRoom>,
) {
    let Some(room) = (**selection).clone() else {
        return;
    };
    let rows = rows.map(Res::into_inner);
    let is_present = rows.is_some_and(|rows| rows.iter().any(|row| row.name == room));
    if !is_present {
        **selection = None;
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::drop_stale_room_selection;

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
        app.insert_resource(ui::discovery::SelectedRoom(
            room.map(core_discovery::RoomName::from),
        ));
        app.add_systems(Update, drop_stale_room_selection);
        app
    }

    fn selected(app: &App) -> Option<core_discovery::RoomName> {
        (**app.world().resource::<ui::discovery::SelectedRoom>()).clone()
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(Some("alpha"), vec![row("alpha")]);
        app.update();
        assert_eq!(
            selected(&app),
            Some(core_discovery::RoomName::from("alpha"))
        );
    }

    #[test]
    fn a_selection_whose_room_vanishes_is_cleared() {
        let mut app = app_with(Some("alpha"), vec![row("beta")]);
        app.update();
        assert_eq!(selected(&app), None);
    }
}
