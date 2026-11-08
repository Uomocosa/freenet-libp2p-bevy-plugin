use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::{InteractionDisabled, Pressed};

use crate::ui;
use crate::ui::discovery::basic::constants::{
    BUTTON_COLOR, BUTTON_LABEL_COLOR, DISABLED_BUTTON_COLOR, DISABLED_BUTTON_LABEL_COLOR,
};
use ui::direct_dial::{AddFriendButton, FriendAddressInput};

type AddButtonParts<'a> = (
    Entity,
    Option<&'a Hovered>,
    Has<InteractionDisabled>,
    &'a mut BackgroundColor,
    Option<&'a Children>,
);

pub fn gate_add_friend(
    input: Single<&EditableText, With<FriendAddressInput>>,
    button: Single<AddButtonParts, With<AddFriendButton>>,
    mut labels: Query<&mut TextColor>,
    mut commands: Commands,
) {
    let typed = input.into_inner().value().to_string();
    let enabled = !typed.trim().is_empty();
    let (entity, hovered, disabled, mut background, children) = button.into_inner();
    let BackgroundColor(existing) = &mut *background;
    let (color, label_color) = if enabled {
        (BUTTON_COLOR, BUTTON_LABEL_COLOR)
    } else {
        (DISABLED_BUTTON_COLOR, DISABLED_BUTTON_LABEL_COLOR)
    };
    if *existing != color {
        *existing = color;
    }
    let mut label_iter = labels
        .iter_many_mut(children.into_iter().flatten())
        .matched();
    while let Some(mut label) = label_iter.fetch_next() {
        let TextColor(current) = &mut *label;
        if *current != label_color {
            *current = label_color;
        }
    }
    match (enabled, hovered.is_some()) {
        (true, false) => {
            commands.entity(entity).insert(Hovered(false));
        }
        (false, true) => {
            commands
                .entity(entity)
                .remove::<Hovered>()
                .remove::<Pressed>();
        }
        _ => {}
    }
    match (enabled, disabled) {
        (true, true) => {
            commands.entity(entity).remove::<InteractionDisabled>();
        }
        (false, false) => {
            commands.entity(entity).insert(InteractionDisabled);
        }
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;
    use bevy::text::EditableText;
    use bevy::ui::InteractionDisabled;

    use super::gate_add_friend;
    use crate::ui;
    use bevy::ui_widgets::{Button, TextInput};

    fn app_with(typed: &str) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.world_mut().spawn((
            ui::direct_dial::FriendAddressInput,
            EditableText::new(typed),
        ));
        app.world_mut().spawn((
            ui::direct_dial::AddFriendButton,
            Hovered(false),
            BackgroundColor(Color::BLACK),
        ));
        app.add_systems(Update, gate_add_friend);
        app
    }

    #[test]
    fn test_usage() {
        let mut named = app_with("  /ip4/1.2.3.4  ");
        named.update();
        let mut query = named
            .world_mut()
            .query_filtered::<(), (With<ui::direct_dial::AddFriendButton>, With<Hovered>)>();
        assert_eq!(query.iter(named.world()).count(), 1);
        let mut blank = app_with("   ");
        blank.update();
        let mut query = blank
            .world_mut()
            .query_filtered::<(), (With<ui::direct_dial::AddFriendButton>, With<Hovered>)>();
        assert_eq!(query.iter(blank.world()).count(), 0);
    }

    #[test]
    fn blank_input_disables_the_button() {
        let mut blank = app_with("   ");
        blank.update();
        let mut query = blank.world_mut().query_filtered::<(), (
            With<ui::direct_dial::AddFriendButton>,
            With<InteractionDisabled>,
        )>();
        assert_eq!(query.iter(blank.world()).count(), 1);
    }

    fn spawn_button(mut commands: Commands) {
        commands.spawn((
            ui::direct_dial::FriendAddressInput,
            EditableText::new(""),
            TextInput,
        ));
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            ui::direct_dial::AddFriendButton,
            Button,
            Hovered::default(),
            Node {
                padding: UiRect::axes(Val::Px(12.0), Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(Color::BLACK),
            children![Text::new("Add Friend")],
        ));
    }

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, spawn_button);
        app.add_systems(Update, gate_add_friend);
    }

    const fn preview_noop(_world: &mut World) {}

    fn type_address(world: &mut World) {
        let mut inputs =
            world.query_filtered::<&mut EditableText, With<ui::direct_dial::FriendAddressInput>>();
        if let Ok(mut input) = inputs.single_mut(world) {
            *input = EditableText::new("/ip4/1.2.3.4/tcp/4001");
        }
    }

    #[test]
    fn gate_add_friend_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("gate_add_friend"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("blank address"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("typed address"),
                    apply: type_address,
                },
            ],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
