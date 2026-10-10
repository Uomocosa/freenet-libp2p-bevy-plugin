use bevy::prelude::*;

use crate::discovery;
use crate::net_id;
use discovery::ui::RoomButton;

pub fn handle_join(
    buttons: Query<(&Interaction, &RoomButton), Changed<Interaction>>,
    snapshot: Res<discovery::Snapshot>,
    mut commands: MessageWriter<discovery::Command>,
) {
    let snapshot = snapshot.into_inner();
    let current = snapshot.room.as_ref().map(|room| &room.name);
    for (interaction, button) in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let room: &net_id::RoomName = button;
        if current == Some(room) {
            tracing::info!(target: "room_lobby", "ui leave room={}", room.as_str());
            commands.write(discovery::Command::Leave);
        } else {
            tracing::info!(target: "room_lobby", "ui join room={}", room.as_str());
            commands.write(discovery::Command::Join(room.clone()));
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::net_id;
    use bevy::prelude::*;

    use super::handle_join;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_message::<discovery::Command>();
        app.init_resource::<discovery::Snapshot>();
        let room = net_id::RoomName::from("alpha");
        app.world_mut().spawn((
            discovery::ui::RoomButton(room.clone()),
            Interaction::Pressed,
        ));
        app.add_systems(Update, handle_join);
        app.update();
        let messages = app.world().resource::<Messages<discovery::Command>>();
        let mut cursor = messages.get_cursor();
        let sent: Vec<_> = cursor.read(messages).cloned().collect();
        assert_eq!(sent, vec![discovery::Command::Join(room)]);
    }
}
