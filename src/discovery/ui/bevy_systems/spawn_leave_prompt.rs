use bevy::prelude::*;

use crate::discovery;
use discovery::ui::basic::constants::{
    BUTTON_COLOR, PANEL_COLOR, PROMPT_BACKDROP_COLOR, YES_COLOR,
};
use discovery::ui::{LeaveDialog, LeaveNoButton, LeavePrompt, LeaveYesButton};

const PROMPT_Z: i32 = 100;

pub fn spawn_leave_prompt(mut commands: Commands) {
    commands.spawn((
        LeavePrompt,
        DespawnOnExit(LeaveDialog::Open),
        GlobalZIndex(PROMPT_Z),
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(PROMPT_BACKDROP_COLOR),
        children![(
            Node {
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: Val::Px(14.0),
                padding: UiRect::all(Val::Px(18.0)),
                ..default()
            },
            BackgroundColor(PANEL_COLOR),
            children![
                Text::new("Are you sure to leave the room?"),
                (
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(12.0),
                        ..default()
                    },
                    children![
                        answer_button(LeaveYesButton, "Yes", YES_COLOR),
                        answer_button(LeaveNoButton, "No", BUTTON_COLOR),
                    ],
                ),
            ],
        )],
    ));
}

// needed helper: Yes and No differ only in marker, label and color
fn answer_button(marker: impl Component, label: &str, color: Color) -> impl Bundle {
    (
        marker,
        Button,
        Node {
            padding: UiRect::axes(Val::Px(18.0), Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(color),
        children![Text::new(label)],
    )
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_leave_prompt;
    use crate::discovery;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, spawn_leave_prompt);
        app.update();
        let mut prompts = app
            .world_mut()
            .query_filtered::<(), With<discovery::ui::LeavePrompt>>();
        assert_eq!(prompts.iter(app.world()).count(), 1);
    }

    fn preview_build(app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_leave_prompt,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn spawn_leave_prompt_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_leave_prompt"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("leave prompt"),
                apply: preview_noop,
            }],
            timeline: None,
        }
        .run(
            &lele_bevy_preview::preview::Config {
                out_dir: out,
                ..Default::default()
            },
            env!("CARGO_PKG_NAME"),
        )
        .expect("preview");
    }
}
