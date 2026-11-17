use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::StreamedRoom;

pub fn clear_stream_on_room_entry(
    mut commands: MessageReader<core_discovery::Command>,
    mut streamed: ResMut<StreamedRoom>,
) {
    for command in commands.read() {
        if matches!(
            command,
            core_discovery::Command::Join(_) | core_discovery::Command::Create(_)
        ) {
            **streamed = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::clear_stream_on_room_entry;

    fn app() -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<core_discovery::Command>();
        app.insert_resource(ui::discovery::StreamedRoom(Some(
            core_discovery::RoomName::from("alpha"),
        )));
        app.add_systems(Update, clear_stream_on_room_entry);
        app
    }

    fn streamed(app: &App) -> Option<core_discovery::RoomName> {
        (**app.world().resource::<ui::discovery::StreamedRoom>()).clone()
    }

    fn write_command(app: &mut App, command: core_discovery::Command) {
        app.world_mut().write_message(command);
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        write_command(
            &mut app,
            core_discovery::Command::Join(core_discovery::RoomName::from("beta")),
        );
        app.update();
        assert_eq!(streamed(&app), None);
    }

    #[test]
    fn creating_a_room_also_closes_the_stream() {
        let mut app = app();
        write_command(
            &mut app,
            core_discovery::Command::Create(core_discovery::RoomName::from("beta")),
        );
        app.update();
        assert_eq!(streamed(&app), None);
    }

    #[test]
    fn watching_a_room_keeps_the_stream_open() {
        let mut app = app();
        write_command(
            &mut app,
            core_discovery::Command::Watch(core_discovery::RoomName::from("beta")),
        );
        app.update();
        assert_eq!(
            streamed(&app),
            Some(core_discovery::RoomName::from("alpha"))
        );
    }
}
