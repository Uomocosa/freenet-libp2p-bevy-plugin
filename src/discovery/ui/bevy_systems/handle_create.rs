use bevy::prelude::*;
use bevy::text::EditableText;

use crate::discovery;
use crate::net_id;
use discovery::ui::{CreateRoomButton, RoomNameInput};

pub fn handle_create(
    buttons: Query<&Interaction, (Changed<Interaction>, With<CreateRoomButton>)>,
    input: Single<&EditableText, With<RoomNameInput>>,
    mut commands: MessageWriter<discovery::Command>,
) {
    let typed = input.into_inner().value().to_string();
    for interaction in &buttons {
        if *interaction != Interaction::Pressed {
            continue;
        }
        let name = typed.trim();
        if name.is_empty() {
            tracing::info!(target: "room_lobby", "ui create ignored: empty room name");
            continue;
        }
        tracing::info!(target: "room_lobby", "ui create room={name}");
        commands.write(discovery::Command::Create(net_id::RoomName(
            name.to_string(),
        )));
    }
}

#[cfg(test)]
mod tests {
    use crate::net_id;
    use bevy::prelude::*;
    use bevy::text::EditableText;

    use super::handle_create;
    use crate::discovery;

    fn sent_after_press(typed: &str) -> Vec<discovery::Command> {
        let mut app = App::new();
        app.add_message::<discovery::Command>();
        app.world_mut()
            .spawn((discovery::ui::RoomNameInput, EditableText::new(typed)));
        app.world_mut()
            .spawn((discovery::ui::CreateRoomButton, Interaction::Pressed));
        app.add_systems(Update, handle_create);
        app.update();
        let messages = app.world().resource::<Messages<discovery::Command>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            sent_after_press(" alpha "),
            vec![discovery::Command::Create(net_id::RoomName(
                "alpha".to_string()
            ))]
        );
        assert_eq!(sent_after_press("   "), Vec::new());
    }
}
