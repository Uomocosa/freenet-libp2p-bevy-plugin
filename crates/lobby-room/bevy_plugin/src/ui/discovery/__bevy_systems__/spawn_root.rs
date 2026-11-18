use crate::ui;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{Button, TextInput};

use ui::discovery::{
    BUTTON_COLOR, CARET_COLOR, CARET_SELECTION_COLOR, CreateRoomButton, HOVER_OUTLINE_WIDTH,
    HoverGlow, HoverOutlineOnly, INPUT_COLOR, LOBBY_PANEL_RIGHT, LOBBY_PANEL_WIDTH, PANEL_COLOR,
    RoomList, RoomNameInput, Root, Screen,
};

pub fn spawn_root(mut commands: Commands) {
    commands
        .spawn((
            Root,
            DespawnOnExit(Screen::Lobby),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(48.0),
                right: Val::Px(LOBBY_PANEL_RIGHT),
                width: Val::Px(LOBBY_PANEL_WIDTH),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(8.0),
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(PANEL_COLOR),
        ))
        .with_children(|root| {
            root.spawn((
                RoomNameInput,
                EditableText::new(""),
                TextInput,
                TabIndex(0),
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
            ));
            root.spawn((
                CreateRoomButton,
                Button,
                InteractionDisabled,
                HoverGlow::default(),
                UiTransform::default(),
                Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE),
                HoverOutlineOnly,
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(BUTTON_COLOR),
                children![(Text::new("+ Create New Room"), TextLayout::no_wrap())],
            ));
            ui::discovery::bevy_systems::spawn_rooms_header(&mut root.reborrow());
            root.spawn((
                RoomList,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
            ));
        });
}

#[cfg(test)]
mod tests {
    use super::spawn_root;
    use bevy::prelude::*;

    #[test]
    fn spawn_root_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_root"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(
                    Startup,
                    (
                        spawn_root,
                        lele_bevy_preview::scene::bevy_systems::center_roots,
                    )
                        .chain(),
                );
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("empty lobby"),
                apply: |_world: &mut World| {},
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
