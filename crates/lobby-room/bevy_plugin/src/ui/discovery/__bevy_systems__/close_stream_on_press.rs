use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{CloseStream, RoomStreamCloseButton};

pub fn close_stream_on_press(
    presses: Query<(), (Added<Pressed>, With<RoomStreamCloseButton>)>,
    mut commands: MessageWriter<CloseStream>,
) {
    if presses.is_empty() {
        return;
    }
    commands.write(CloseStream);
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::close_stream_on_press;

    fn app() -> App {
        let mut app = App::new();
        app.add_message::<ui::discovery::CloseStream>();
        app.add_systems(Update, close_stream_on_press);
        app
    }

    fn press(app: &mut App) {
        app.world_mut()
            .spawn((ui::discovery::RoomStreamCloseButton, Pressed));
    }

    fn close_requests(app: &App) -> usize {
        let messages = app
            .world()
            .resource::<Messages<ui::discovery::CloseStream>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).count()
    }

    #[test]
    fn test_usage() {
        let mut app = app();
        press(&mut app);
        app.update();
        assert_eq!(close_requests(&app), 1);
    }

    #[test]
    fn another_press_writes_nothing() {
        let mut app = app();
        app.world_mut().spawn(ui::discovery::RoomStreamActionSlot);
        app.update();
        assert_eq!(close_requests(&app), 0);
    }
}
