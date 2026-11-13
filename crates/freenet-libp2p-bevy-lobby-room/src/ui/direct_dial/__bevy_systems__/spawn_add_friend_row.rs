use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{Button, TextInput};

use ui::direct_dial::{AddFriendButton, FriendAddressInput};
use ui::discovery::{
    BUTTON_COLOR, BUTTON_LABEL_COLOR, CARET_COLOR, CARET_SELECTION_COLOR, HOVER_OUTLINE_WIDTH,
    HoverGlow, INPUT_COLOR,
};

pub fn spawn_add_friend_row(row: &mut ChildSpawnerCommands) {
    row.spawn((Node {
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        column_gap: Val::Px(6.0),
        ..default()
    },))
        .with_children(|row| {
            row.spawn((
                FriendAddressInput,
                EditableText::new(""),
                TextInput,
                TextCursorStyle {
                    color: CARET_COLOR,
                    selection_color: CARET_SELECTION_COLOR,
                    unfocused_selection_color: Color::NONE,
                    selected_text_color: None,
                    selection_radius: 0.0,
                },
                Hovered::default(),
                Node {
                    flex_grow: 1.0,
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(INPUT_COLOR),
            ));
            row.spawn((
                AddFriendButton,
                Button,
                InteractionDisabled,
                Hovered::default(),
                HoverGlow::default(),
                UiTransform::default(),
                Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE),
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(BUTTON_COLOR),
                children![(Text::new("+"), TextColor(BUTTON_LABEL_COLOR),)],
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::spawn_add_friend_row;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_empty().with_children(|panel| {
                spawn_add_friend_row(panel);
            });
        });
        app.update();
        let mut inputs = app
            .world_mut()
            .query_filtered::<(), With<ui::direct_dial::FriendAddressInput>>();
        assert_eq!(inputs.iter(app.world()).count(), 1);
        let mut buttons = app
            .world_mut()
            .query_filtered::<(), With<ui::direct_dial::AddFriendButton>>();
        assert_eq!(buttons.iter(app.world()).count(), 1);
    }

    fn spawn_row(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands
            .spawn((
                ChildOf(root),
                Node {
                    width: Val::Px(240.0),
                    ..default()
                },
                BackgroundColor(ui::constants::DRAWER_PANEL_COLOR),
            ))
            .with_children(|panel| {
                spawn_add_friend_row(panel);
            });
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_build(app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_row,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
    }

    #[test]
    fn spawn_add_friend_row_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_add_friend_row"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("input and add button"),
                apply: preview_noop,
            }],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
