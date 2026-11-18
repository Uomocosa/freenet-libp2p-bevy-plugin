use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::Pressed;

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
        commands.write(core_discovery::Command::Create(
            core_discovery::RoomName::from(name),
        ));
    }
}
