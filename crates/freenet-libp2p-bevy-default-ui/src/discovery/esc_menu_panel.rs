use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use crate::direct_dial;
use crate::discovery;
use discovery::{
    BUTTON_COLOR, BUTTON_LABEL_COLOR, HOVER_OUTLINE_WIDTH, HoverGlow, MENU_BUTTON_RADIUS,
    MENU_HINT_FONT_SIZE, MENU_LABEL_FONT_SIZE,
};

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct EscMenuPanel;

#[rustfmt::skip]
impl EscMenuPanel {
    pub(crate) fn button(marker: impl Component, label: &str, hint: &str) -> impl Bundle {
        (
            marker,
            Button,
            Hovered::default(),
            HoverGlow::default(),
            UiTransform::default(),
            Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE),
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                border_radius: BorderRadius::all(Val::Px(MENU_BUTTON_RADIUS)),
                align_items: AlignItems::Center,
                column_gap: Val::Px(10.0),
                ..default()
            },
            BackgroundColor(BUTTON_COLOR),
            children![
                (
                    Text::new(label),
                    TextColor(BUTTON_LABEL_COLOR),
                    TextFont {
                        font_size: FontSize::Px(MENU_LABEL_FONT_SIZE),
                        ..default()
                    },
                ),
                (Node {
                    flex_grow: 1.0,
                    ..default()
                },),
                (
                    Text::new(hint),
                    TextColor(direct_dial::DRAWER_DIM_COLOR),
                    TextFont {
                        font_size: FontSize::Px(MENU_HINT_FONT_SIZE),
                        ..default()
                    },
                ),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::EscMenuPanel;
    use crate::basic::constants::DRAWER_PANEL_COLOR;
    use crate::discovery;
    use discovery::EscMenuFriendsButton;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            commands.spawn_empty().with_children(|panel| {
                panel.spawn(EscMenuPanel::button(EscMenuFriendsButton, "Friends", "Tab"));
            });
        });
        app.update();
        let mut buttons = app
            .world_mut()
            .query_filtered::<(), With<EscMenuFriendsButton>>();
        assert_eq!(buttons.iter(app.world()).count(), 1);
    }

    fn spawn_button(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands
            .spawn((
                ChildOf(root),
                EscMenuPanel,
                Node {
                    width: Val::Px(discovery::MENU_WIDTH),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(14.0)),
                    border_radius: BorderRadius::all(Val::Px(discovery::MENU_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(DRAWER_PANEL_COLOR),
            ))
            .with_children(|panel| {
                panel.spawn(EscMenuPanel::button(EscMenuFriendsButton, "Friends", "Tab"));
            });
    }

    const fn preview_noop(_world: &mut World) {}

    fn preview_build(app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_button,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
    }

    #[test]
    fn esc_menu_panel_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("esc_menu_panel"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("friends row"),
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
