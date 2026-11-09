use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::Pressed;

use crate::discovery;
use discovery::{CreateRoomButton, RoomNameInput};
use freenet_libp2p_bevy_core::discovery as core_discovery;
use freenet_libp2p_bevy_core::identity;

pub fn handle_create(
    buttons: Query<(), (Added<Pressed>, With<CreateRoomButton>)>,
    input: Single<&EditableText, With<RoomNameInput>>,
    mut commands: MessageWriter<core_discovery::Command>,
) {
    let typed = input.into_inner().value().to_string();
    for () in &buttons {
        let name = typed.trim();
        if name.is_empty() {
            tracing::error!(target: "discovery", "ui create rejected: empty room name");
            continue;
        }
        tracing::info!(target: "discovery", "ui create room={name}");
        commands.write(core_discovery::Command::Create(identity::RoomName::from(
            name,
        )));
    }
}

#[cfg(test)]
mod tests {
    use crate::discovery;
    use bevy::prelude::*;
    use bevy::text::EditableText;
    use bevy::ui::Pressed;
    use freenet_libp2p_bevy_core::identity;

    use super::handle_create;
    use freenet_libp2p_bevy_core::discovery as core_discovery;

    fn sent_after_press(typed: &str) -> Vec<core_discovery::Command> {
        let mut app = App::new();
        app.add_message::<core_discovery::Command>();
        app.world_mut()
            .spawn((discovery::RoomNameInput, EditableText::new(typed)));
        app.world_mut()
            .spawn((discovery::CreateRoomButton, Pressed));
        app.add_systems(Update, handle_create);
        app.update();
        let messages = app.world().resource::<Messages<core_discovery::Command>>();
        let mut cursor = messages.get_cursor();
        cursor.read(messages).cloned().collect()
    }

    #[test]
    fn test_usage() {
        assert_eq!(
            sent_after_press(" alpha "),
            vec![core_discovery::Command::Create(identity::RoomName::from(
                "alpha"
            ))]
        );
        assert_eq!(sent_after_press("   "), Vec::new());
    }
}
