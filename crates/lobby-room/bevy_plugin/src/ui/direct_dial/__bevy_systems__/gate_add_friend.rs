use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::EditableText;
use bevy::ui::{InteractionDisabled, Pressed};

use ui::direct_dial::{AddFriendButton, FriendAddressInput};
use ui::discovery::{
    BUTTON_COLOR, BUTTON_LABEL_COLOR, DISABLED_BUTTON_COLOR, DISABLED_BUTTON_LABEL_COLOR,
};

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
    use crate::ui;
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;
    use bevy::text::EditableText;
    use bevy::ui_widgets::{Button, TextInput};

    use super::gate_add_friend;

    #[test]
    fn gate_add_friend_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("gate_add_friend"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(Startup, |mut commands: Commands| {
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
                });
                app.add_systems(Update, gate_add_friend);
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("blank address"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("typed address"),
                    apply: |world: &mut World| {
                        let mut inputs = world.query_filtered::<&mut EditableText, With<
                            ui::direct_dial::FriendAddressInput,
                        >>();
                        if let Ok(mut input) = inputs.single_mut(world) {
                            *input = EditableText::new("/ip4/1.2.3.4/tcp/4001");
                        }
                    },
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
