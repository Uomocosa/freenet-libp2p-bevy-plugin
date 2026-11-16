use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{RoomButton, SelectedRoom};

pub fn lock_room_selection(
    buttons: Query<&RoomButton, Added<Pressed>>,
    mut selection: ResMut<SelectedRoom>,
) {
    for button in &buttons {
        selection.room = Some((**button).clone());
        selection.is_locked = true;
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::lock_room_selection;

    fn sent_commands(app: &App) -> Vec<core_discovery::Command> {
        let messages = app.world().resource::<Messages<core_discovery::Command>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<core_discovery::Command>();
        app.init_resource::<ui::discovery::SelectedRoom>();
        let room = core_discovery::RoomName::from("alpha");
        app.world_mut()
            .spawn((ui::discovery::RoomButton(room.clone()), Pressed));
        app.add_systems(
            Update,
            (
                lock_room_selection,
                ui::discovery::bevy_systems::handle_join,
            ),
        );
        app.update();
        let selection = app.world().resource::<ui::discovery::SelectedRoom>();
        assert_eq!(selection.room, Some(room));
        assert!(selection.is_locked);
    }

    #[test]
    fn a_single_click_selects_without_joining() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_message::<core_discovery::Command>();
        app.init_resource::<ui::discovery::SelectedRoom>();
        app.world_mut().spawn((
            ui::discovery::RoomButton(core_discovery::RoomName::from("alpha")),
            Pressed,
        ));
        app.add_systems(
            Update,
            (
                lock_room_selection,
                ui::discovery::bevy_systems::handle_join,
            ),
        );
        app.update();
        assert_eq!(
            app.world().resource::<ui::discovery::SelectedRoom>().room,
            Some(core_discovery::RoomName::from("alpha"))
        );
        assert_eq!(sent_commands(&app), Vec::new());
    }
}
