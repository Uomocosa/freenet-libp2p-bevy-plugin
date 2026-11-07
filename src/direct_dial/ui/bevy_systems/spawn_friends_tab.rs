use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{Button, TextInput};

use crate::direct_dial;
use crate::discovery::ui::basic::constants::{
    BUTTON_COLOR, CARET_COLOR, CARET_SELECTION_COLOR, INPUT_COLOR, PANEL_COLOR,
};

pub fn spawn_friends_tab(mut commands: Commands) {
    commands.spawn((
        direct_dial::ui::FriendsTab,
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(48.0),
            left: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(PANEL_COLOR),
        children![
            Text::new("Friends"),
            (
                direct_dial::ui::OwnAddressText,
                Text::new("Your address: waiting for network..."),
            ),
            (
                direct_dial::ui::FriendAddressInput,
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
                    min_width: Val::Px(200.0),
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(INPUT_COLOR),
            ),
            (
                direct_dial::ui::AddFriendButton,
                Button,
                InteractionDisabled,
                Hovered::default(),
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(BUTTON_COLOR),
                children![Text::new("+ Add Friend")],
            ),
            (direct_dial::ui::FriendErrorText, Text::new("")),
            (
                direct_dial::ui::FriendList,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
            ),
        ],
    ));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_friends_tab;
    use crate::direct_dial;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, spawn_friends_tab);
        app.update();
        let mut tabs = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::ui::FriendsTab>>();
        assert_eq!(tabs.iter(app.world()).count(), 1);
        let mut inputs = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::ui::FriendAddressInput>>();
        assert_eq!(inputs.iter(app.world()).count(), 1);
    }

    fn preview_build(app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_friends_tab,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn spawn_friends_tab_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_friends_tab"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("empty friends tab"),
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
