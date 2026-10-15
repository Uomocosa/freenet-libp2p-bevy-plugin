use bevy::prelude::*;
use bevy::text::EditableText;

use crate::discovery;
use discovery::ui::{CreateRoomButton, RoomNameInput};

const ENABLED_COLOR: Color = Color::srgb(0.20, 0.24, 0.32);
const DISABLED_COLOR: Color = Color::srgb(0.13, 0.13, 0.14);
const LABEL_COLOR: Color = Color::WHITE;
const DISABLED_LABEL_COLOR: Color = Color::srgb(0.38, 0.38, 0.40);

type CreateButtonParts<'a> = (
    Entity,
    Option<&'a Interaction>,
    &'a mut BackgroundColor,
    Option<&'a Children>,
);

// needed helper: keeps the Create button clickable only while the room name is non-empty
pub fn gate_create(
    input: Single<&EditableText, With<RoomNameInput>>,
    button: Single<CreateButtonParts, With<CreateRoomButton>>,
    mut labels: Query<&mut TextColor>,
    mut commands: Commands,
) {
    let typed = input.into_inner().value().to_string();
    let enabled = !typed.trim().is_empty();
    let (entity, interaction, mut background, children) = button.into_inner();
    let BackgroundColor(existing) = &mut *background;
    let (color, label_color) = if enabled {
        (ENABLED_COLOR, LABEL_COLOR)
    } else {
        (DISABLED_COLOR, DISABLED_LABEL_COLOR)
    };
    if *existing != color {
        *existing = color;
    }
    let mut label_iter = labels.iter_many_mut(children.into_iter().flatten());
    while let Some(mut label) = label_iter.fetch_next() {
        let TextColor(current) = &mut *label;
        if *current != label_color {
            *current = label_color;
        }
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

    fn spawn_button(mut commands: Commands) {
        commands.spawn((discovery::ui::RoomNameInput, EditableText::new("")));
        commands.spawn((
            discovery::ui::CreateRoomButton,
            Button,
            Node {
                width: Val::Px(160.0),
                height: Val::Px(32.0),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            children![Text::new("Create New Room")],
        ));
    }

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, spawn_button);
        app.add_systems(Update, gate_create);
    }

    const fn preview_noop(_world: &mut World) {}

    fn type_name(world: &mut World) {
        let mut inputs =
            world.query_filtered::<&mut EditableText, With<discovery::ui::RoomNameInput>>();
        if let Ok(mut input) = inputs.single_mut(world) {
            *input = EditableText::new("alpha");
        }
    }

    #[test]
    fn gate_create_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::run(
            &lele_bevy_preview::scene::Scene {
                name: String::from("gate_create"),
                kind: lele_bevy_preview::scene::Kind::System,
                build: preview_build,
                states: vec![
                    lele_bevy_preview::scene::State {
                        label: String::from("blank name"),
                        apply: preview_noop,
                    },
                    lele_bevy_preview::scene::State {
                        label: String::from("typed name"),
                        apply: type_name,
                    },
                ],
                timeline: None,
            },
            &lele_bevy_preview::preview::Config {
                out_dir: out,
                ..Default::default()
            },
            env!("CARGO_PKG_NAME"),
        )
        .expect("preview");
    }
}
