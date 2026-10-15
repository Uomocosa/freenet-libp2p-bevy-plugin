use bevy::prelude::*;
use bevy::text::EditableText;

use crate::discovery;
use discovery::ui::{CreateRoomButton, RoomNameInput};

const ENABLED_COLOR: Color = Color::srgb(0.20, 0.24, 0.32);
const DISABLED_COLOR: Color = Color::srgb(0.14, 0.15, 0.17);

// needed helper: keeps the Create button clickable only while the room name is non-empty
pub fn gate_create(
    input: Single<&EditableText, With<RoomNameInput>>,
    button: Single<(Entity, Option<&Interaction>, &mut BackgroundColor), With<CreateRoomButton>>,
    mut commands: Commands,
) {
    let typed = input.into_inner().value().to_string();
    let enabled = !typed.trim().is_empty();
    let (entity, interaction, mut background) = button.into_inner();
    let BackgroundColor(existing) = &mut *background;
    let color = if enabled {
        ENABLED_COLOR
    } else {
        DISABLED_COLOR
    };
    if *existing != color {
        *existing = color;
    }
    match (enabled, interaction.is_some()) {
        (true, false) => {
            commands.entity(entity).insert(Interaction::None);
        }
        (false, true) => {
            commands.entity(entity).remove::<Interaction>();
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::text::EditableText;

    use super::gate_create;
    use crate::discovery;

    fn app_with(typed: &str) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut()
            .spawn((discovery::ui::RoomNameInput, EditableText::new(typed)));
        app.world_mut().spawn((
            discovery::ui::CreateRoomButton,
            Interaction::None,
            BackgroundColor(Color::BLACK),
        ));
        app.add_systems(Update, gate_create);
        app
    }

    fn clickable(app: &mut App) -> usize {
        let mut query = app
            .world_mut()
            .query_filtered::<(), (With<discovery::ui::CreateRoomButton>, With<Interaction>)>();
        query.iter(app.world()).count()
    }

    #[test]
    fn test_usage() {
        let mut named = app_with("alpha");
        named.update();
        assert_eq!(clickable(&mut named), 1);

        let mut blank = app_with("   ");
        blank.update();
        assert_eq!(clickable(&mut blank), 0);
    }
}
