use crate::ui;
use bevy::input_focus::tab_navigation::TabIndex;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{EditableText, TextCursorStyle};
use bevy::ui::InteractionDisabled;
use bevy::ui_widgets::{Button, TextInput};

use ui::discovery::{BUTTON_COLOR, CARET_COLOR, CARET_SELECTION_COLOR, INPUT_COLOR, PANEL_COLOR};
use ui::discovery::{
    CreateRoomButton, HOVER_OUTLINE_WIDTH, HoverGlow, HoverOutlineOnly, RoomList, RoomNameInput,
    Root, Screen,
};

pub fn spawn_root(mut commands: Commands) {
    commands.spawn((
        Root,
        DespawnOnExit(Screen::Lobby),
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(48.0),
            right: Val::Px(12.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(8.0),
            padding: UiRect::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(PANEL_COLOR),
        children![
            (
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
                HoverGlow::default(),
                UiTransform::default(),
                Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE),
                HoverOutlineOnly,
                Node {
                    min_width: Val::Px(200.0),
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(INPUT_COLOR),
            ),
            (
                CreateRoomButton,
                Button,
                InteractionDisabled,
                HoverGlow::default(),
                UiTransform::default(),
                Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE),
                Node {
                    padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                    ..default()
                },
                BackgroundColor(BUTTON_COLOR),
                children![Text::new("+ Create New Room")],
            ),
            (
                RoomList,
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
    use crate::ui;
    use bevy::input_focus::{InputFocus, InputFocusPlugin, pointer_focus::PointerFocusPlugin};
    use bevy::picking::backend::HitData;
    use bevy::picking::events::{Pointer, PointerPress};
    use bevy::picking::pointer::{Location, PointerButton, PointerId};
    use bevy::prelude::*;
    use bevy::window::PrimaryWindow;

    use super::spawn_root;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, spawn_root);
        app.update();
        let mut lists = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomList>>();
        assert_eq!(lists.iter(app.world()).count(), 1);
        let mut inputs = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomNameInput>>();
        assert_eq!(inputs.iter(app.world()).count(), 1);
    }

    fn preview_build(app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_root,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn spawn_root_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_root"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("empty lobby"),
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

    #[test]
    fn a_primary_press_focuses_the_room_name_input() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InputFocusPlugin, PointerFocusPlugin))
            .add_systems(Startup, spawn_root);
        let window = app
            .world_mut()
            .spawn((Window::default(), PrimaryWindow))
            .id();
        app.update();
        let input = app
            .world_mut()
            .query_filtered::<Entity, With<ui::discovery::RoomNameInput>>()
            .single(app.world())
            .expect("one room name input");
        let location = Location {
            target: bevy::camera::NormalizedRenderTarget::Window(
                bevy::window::WindowRef::Entity(window)
                    .normalize(Some(window))
                    .expect("window target"),
            ),
            position: Vec2::ZERO,
        };
        app.world_mut().trigger(PointerPress {
            entity: input,
            pointer: Pointer::new(PointerId::Mouse, location),
            button: PointerButton::Primary,
            hit: HitData::new(Entity::PLACEHOLDER, 0.0, None, None),
            count: 1,
        });
        app.update();
        assert_eq!(app.world().resource::<InputFocus>().get(), Some(input));
    }
}
