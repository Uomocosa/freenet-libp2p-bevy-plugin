use bevy::prelude::*;
use bevy::ui::Pressed;

use crate::discovery;
use crate::identity;
use crate::ui;
use ui::discovery::RoomButton;

pub fn handle_join(
    buttons: Query<&RoomButton, Added<Pressed>>,
    status: Res<discovery::Status>,
    mut commands: MessageWriter<discovery::Command>,
) {
    let status = status.into_inner();
    let current = status.room.as_ref().map(|room| &room.name);
    for button in &buttons {
        let room: &identity::RoomName = button;
        if current == Some(room) {
            tracing::info!(target: "discovery", "ui leave room={}", room.as_str());
            commands.write(discovery::Command::Leave);
        } else {
            tracing::info!(target: "discovery", "ui join room={}", room.as_str());
            commands.write(discovery::Command::Join(room.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::identity;
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::handle_join;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<discovery::Command>();
        app.init_resource::<discovery::Status>();
        let room = identity::RoomName::from("alpha");
        app.world_mut()
            .spawn((ui::discovery::RoomButton(room.clone()), Pressed));
        app.add_systems(Update, handle_join);
        app.update();
        let messages = app.world().resource::<Messages<discovery::Command>>();
        let mut cursor = messages.get_cursor();
        let sent: Vec<_> = cursor.read(messages).cloned().collect();
        assert_eq!(sent, vec![discovery::Command::Join(room)]);
    }
}
