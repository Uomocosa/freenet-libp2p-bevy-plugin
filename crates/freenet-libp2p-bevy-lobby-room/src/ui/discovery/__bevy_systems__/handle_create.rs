use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::Pressed;

use freenet_libp2p_bevy_core::identity;
use ui::discovery::{CreateRoomButton, RoomNameInput};

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
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;
    use bevy::text::EditableText;
    use bevy::ui::Pressed;
    use freenet_libp2p_bevy_core::identity;

    use super::handle_create;

    fn sent_after_press(typed: &str) -> Vec<core_discovery::Command> {
        let mut app = App::new();
        app.add_message::<core_discovery::Command>();
        app.world_mut()
            .spawn((ui::discovery::RoomNameInput, EditableText::new(typed)));
        app.world_mut()
            .spawn((ui::discovery::CreateRoomButton, Pressed));
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
