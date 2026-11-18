use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use ui::direct_dial::OwnAddressChip;
use ui::discovery::{ACCENT_COLOR, HOVER_OUTLINE_WIDTH, HoverGlow, INPUT_COLOR};

pub fn spawn_address_chip(chip: &mut ChildSpawnerCommands) {
    chip.spawn((
        OwnAddressChip,
        Button,
        Hovered::default(),
        HoverGlow::default(),
        UiTransform::default(),
        Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(INPUT_COLOR),
        children![
            (
                ui::direct_dial::OwnAddressText,
                Text::new("waiting for network...")
            ),
            (Node {
                flex_grow: 1.0,
                ..default()
            },),
            (
                Text::new("copy"),
                TextColor(ACCENT_COLOR),
                TextFont {
                    font_size: FontSize::Px(11.0),
                    ..default()
                },
            ),
        ],
    ));
}

#[cfg(test)]
mod tests {
    use super::spawn_address_chip;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn spawn_address_chip_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_address_chip"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(
                    Startup,
                    (
                        |mut commands: Commands| {
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
                                    spawn_address_chip(panel);
                                });
                        },
                        lele_bevy_preview::scene::bevy_systems::center_roots,
                    )
                        .chain(),
                );
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("address chip"),
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
