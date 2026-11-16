use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::EnterRoomButton;

pub fn handle_join(
    buttons: Query<&EnterRoomButton, Added<Pressed>>,
    mut commands: MessageWriter<core_discovery::Command>,
) {
    for button in &buttons {
        let room: &core_discovery::RoomName = button;
        tracing::info!(target: "discovery", "ui join room={}", room.as_str());
        commands.write(core_discovery::Command::Join(room.clone()));
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::handle_join;

    fn sent(app: &App) -> Vec<core_discovery::Command> {
        let messages = app.world().resource::<Messages<core_discovery::Command>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<core_discovery::Command>();
        let room = core_discovery::RoomName::from("alpha");
        app.world_mut()
            .spawn((ui::discovery::EnterRoomButton(room.clone()), Pressed));
        app.add_systems(Update, handle_join);
        app.update();
        assert_eq!(sent(&app), vec![core_discovery::Command::Join(room)]);
    }

    #[test]
    fn a_row_press_does_not_join() {
        let mut app = App::new();
        app.add_message::<core_discovery::Command>();
        app.world_mut().spawn((
            ui::discovery::RoomButton(core_discovery::RoomName::from("alpha")),
            Pressed,
        ));
        app.add_systems(Update, handle_join);
        app.update();
        assert_eq!(sent(&app), Vec::new());
    }
}
