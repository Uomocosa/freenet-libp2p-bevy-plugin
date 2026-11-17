use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::{SelectedRoom, StreamedRoom};

pub fn watch_selected_room(
    selection: Res<SelectedRoom>,
    streamed: Res<StreamedRoom>,
    mut commands: MessageWriter<core_discovery::Command>,
    mut watched_room: Local<Option<core_discovery::RoomName>>,
    mut watched_streamed_room: Local<Option<core_discovery::RoomName>>,
) {
    let selected_room = (**selection.into_inner()).clone();
    let streamed_room = (**streamed.into_inner()).clone();
    if *watched_streamed_room != streamed_room {
        (*watched_streamed_room).clone_from(&streamed_room);
        if let Some(room) = &streamed_room {
            commands.write(core_discovery::Command::Watch(room.clone()));
        }
    }
    if *watched_room != selected_room {
        (*watched_room).clone_from(&selected_room);
        if selected_room.as_ref() != streamed_room.as_ref()
            && let Some(room) = &selected_room
        {
            commands.write(core_discovery::Command::Watch(room.clone()));
        }
    }
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
        app.insert_resource(ui::discovery::StreamedRoom::default());
        app.add_systems(Update, watch_selected_room);
        app
    }

    fn select(app: &mut App, room: Option<&str>) {
        **app
            .world_mut()
            .resource_mut::<ui::discovery::SelectedRoom>() =
            room.map(core_discovery::RoomName::from);
    }

    fn stream(app: &mut App, room: Option<&str>) {
        **app
            .world_mut()
            .resource_mut::<ui::discovery::StreamedRoom>() =
            room.map(core_discovery::RoomName::from);
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
    fn re_selecting_after_clearing_emits_a_new_watch() {
        let mut app = app();
        select(&mut app, Some("alpha"));
        app.update();
        let _ = sent(&mut app);
        select(&mut app, None);
        app.update();
        let _ = sent(&mut app);
        select(&mut app, Some("alpha"));
        app.update();
        let room = core_discovery::RoomName::from("alpha");
        assert_eq!(sent(&mut app), vec![core_discovery::Command::Watch(room)]);
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

    #[test]
    fn streaming_a_room_watches_it_too() {
        let mut app = app();
        stream(&mut app, Some("alpha"));
        app.update();
        let room = core_discovery::RoomName::from("alpha");
        assert_eq!(sent(&mut app), vec![core_discovery::Command::Watch(room)]);
    }

    #[test]
    fn watching_the_streamed_room_and_a_hovered_room_emits_both() {
        let mut app = app();
        stream(&mut app, Some("alpha"));
        select(&mut app, Some("beta"));
        app.update();
        assert_eq!(
            sent(&mut app),
            vec![
                core_discovery::Command::Watch(core_discovery::RoomName::from("alpha")),
                core_discovery::Command::Watch(core_discovery::RoomName::from("beta")),
            ]
        );
    }
}
