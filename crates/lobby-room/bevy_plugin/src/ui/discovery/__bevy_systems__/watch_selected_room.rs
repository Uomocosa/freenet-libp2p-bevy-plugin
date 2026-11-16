use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::SelectedRoom;

pub fn watch_selected_room(
    selection: Res<SelectedRoom>,
    mut commands: MessageWriter<core_discovery::Command>,
    mut watched_room: Local<Option<core_discovery::RoomName>>,
) {
    let selection = selection.into_inner();
    let Some(room) = selection.room.clone() else {
        return;
    };
    if watched_room.as_ref() == Some(&room) {
        return;
    }
    *watched_room = Some(room.clone());
    commands.write(core_discovery::Command::Watch(room));
}

#[cfg(test)]
mod tests {
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    use super::watch_selected_room;

    fn app() -> App {
        let mut app = App::new();
        app.add_message::<core_discovery::Command>();
        app.insert_resource(ui::discovery::SelectedRoom::default());
        app.add_systems(Update, watch_selected_room);
        app
    }

    fn select(app: &mut App, room: Option<&str>) {
        app.world_mut()
            .resource_mut::<ui::discovery::SelectedRoom>()
            .room = room.map(core_discovery::RoomName::from);
    }

    fn sent(app: &mut App) -> Vec<core_discovery::Command> {
        app.world_mut()
            .resource_mut::<Messages<core_discovery::Command>>()
            .drain()
            .collect()
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        select(&mut app, Some("alpha"));
        app.update();
        let room = core_discovery::RoomName::from("alpha");
        assert_eq!(sent(&mut app), vec![core_discovery::Command::Watch(room)]);
    }

    #[test]
    fn re_selecting_the_same_room_does_not_emit_again() {
        let mut app = app();
        select(&mut app, Some("alpha"));
        app.update();
        let _ = sent(&mut app);
        select(&mut app, Some("alpha"));
        app.update();
        assert_eq!(sent(&mut app), Vec::new());
    }

    #[test]
    fn clearing_the_preview_keeps_the_last_watched_room() {
        let mut app = app();
        select(&mut app, Some("alpha"));
        app.update();
        let _ = sent(&mut app);
        select(&mut app, None);
        app.update();
        let _ = sent(&mut app);
        select(&mut app, Some("alpha"));
        app.update();
        assert_eq!(sent(&mut app), Vec::new());
    }

    #[test]
    fn selecting_a_different_room_emits_a_new_watch() {
        let mut app = app();
        select(&mut app, Some("alpha"));
        app.update();
        let _ = sent(&mut app);
        select(&mut app, Some("beta"));
        app.update();
        let room = core_discovery::RoomName::from("beta");
        assert_eq!(sent(&mut app), vec![core_discovery::Command::Watch(room)]);
    }
}
