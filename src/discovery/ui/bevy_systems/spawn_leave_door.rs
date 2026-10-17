use bevy::prelude::*;

use crate::discovery;
use discovery::ui::basic::constants::{
    DOOR_FRAME_COLOR, DOOR_HEIGHT, DOOR_KNOB_COLOR, DOOR_LIGHT_COLOR, DOOR_PANEL_COLOR, DOOR_WIDTH,
};
use discovery::ui::{DoorLabel, DoorOpening, DoorPanel, LeaveDoor, LeaveDoorRoot, Screen};

pub fn spawn_leave_door(mut commands: Commands) {
    commands.spawn((
        LeaveDoorRoot,
        DespawnOnExit(Screen::InRoom),
        Node {
            position_type: PositionType::Absolute,
            right: Val::Px(16.0),
            bottom: Val::Px(16.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(10.0),
            ..default()
        },
        children![
            (
                DoorLabel,
                Text::new("Leave Room\n(Esc)"),
                TextColor(Color::WHITE.with_alpha(0.0)),
            ),
            (
                LeaveDoor,
                DoorOpening::default(),
                Button,
                Node {
                    width: Val::Px(DOOR_WIDTH),
                    height: Val::Px(DOOR_HEIGHT),
                    border: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                BorderColor::all(DOOR_FRAME_COLOR),
                BackgroundColor(DOOR_LIGHT_COLOR),
                children![(
                    DoorPanel,
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(DOOR_PANEL_COLOR),
                    children![(
                        Node {
                            position_type: PositionType::Absolute,
                            right: Val::Px(5.0),
                            top: Val::Percent(48.0),
                            width: Val::Px(6.0),
                            height: Val::Px(6.0),
                            ..default()
                        },
                        BackgroundColor(DOOR_KNOB_COLOR),
                    )],
                )],
            ),
        ],
    ));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_leave_door;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, spawn_leave_door);
        app.update();
        let mut doors = app
            .world_mut()
            .query_filtered::<(), With<discovery::ui::LeaveDoor>>();
        assert_eq!(doors.iter(app.world()).count(), 1);
    }

    fn preview_build(app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_leave_door,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn spawn_leave_door_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::run(
            &lele_bevy_preview::scene::Scene {
                name: String::from("spawn_leave_door"),
                kind: lele_bevy_preview::scene::Kind::System,
                build: preview_build,
                states: vec![lele_bevy_preview::scene::State {
                    label: String::from("closed door"),
                    apply: preview_noop,
                }],
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
