use bevy::prelude::*;

use crate::basic::constants::DIM_TEXT_COLOR;
use crate::direct_dial;
use direct_dial::{
    DRAWER_ERROR_COLOR, DRAWER_SECTION_LABEL_FONT_SIZE, FriendsCountText, FriendsDrawerPanel,
};

pub fn spawn_drawer_sections(
    panel: Single<Entity, With<FriendsDrawerPanel>>,
    mut commands: Commands,
) {
    let panel = panel.into_inner();
    commands.entity(panel).with_children(|panel| {
        direct_dial::bevy_systems::spawn_drawer_header(panel);
        panel.spawn((
            Text::new("YOUR ADDRESS"),
            TextFont {
                font_size: FontSize::Px(DRAWER_SECTION_LABEL_FONT_SIZE),
                ..default()
            },
            TextColor(DIM_TEXT_COLOR),
        ));
        direct_dial::bevy_systems::spawn_address_chip(panel);
        panel.spawn((
            Text::new("ADD A FRIEND"),
            TextFont {
                font_size: FontSize::Px(DRAWER_SECTION_LABEL_FONT_SIZE),
                ..default()
            },
            TextColor(DIM_TEXT_COLOR),
        ));
        direct_dial::bevy_systems::spawn_add_friend_row(panel);
        panel.spawn((
            direct_dial::FriendErrorText,
            Text::new(""),
            TextColor(DRAWER_ERROR_COLOR),
            TextFont {
                font_size: FontSize::Px(11.0),
                ..default()
            },
        ));
        panel.spawn((
            FriendsCountText,
            Text::new("FRIENDS  0 online / 0"),
            TextColor(DIM_TEXT_COLOR),
            TextFont {
                font_size: FontSize::Px(DRAWER_SECTION_LABEL_FONT_SIZE),
                ..default()
            },
        ));
        panel.spawn((
            direct_dial::FriendList,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(6.0),
                ..default()
            },
        ));
    });
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_drawer_sections;
    use crate::basic::constants::DRAWER_PANEL_COLOR;
    use crate::direct_dial;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, (spawn_panel, spawn_drawer_sections).chain());
        app.update();
        let mut inputs = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::FriendAddressInput>>();
        assert_eq!(inputs.iter(app.world()).count(), 1);
        let mut chips = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::OwnAddressChip>>();
        assert_eq!(chips.iter(app.world()).count(), 1);
        let mut lists = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::FriendList>>();
        assert_eq!(lists.iter(app.world()).count(), 1);
        let mut errors = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::FriendErrorText>>();
        assert_eq!(errors.iter(app.world()).count(), 1);
    }

    fn spawn_panel(mut commands: Commands) {
        commands.spawn((
            direct_dial::FriendsDrawerPanel,
            Node {
                width: Val::Px(264.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(10.0),
                padding: UiRect::all(Val::Px(14.0)),
                ..default()
            },
            BackgroundColor(DRAWER_PANEL_COLOR),
        ));
    }

    fn preview_build(app: &mut App) {
        app.add_systems(
            Startup,
            (
                spawn_panel,
                spawn_drawer_sections,
                lele_bevy_preview::scene::bevy_systems::center_roots,
            )
                .chain(),
        );
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn spawn_drawer_sections_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_drawer_sections"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("drawer chrome"),
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
