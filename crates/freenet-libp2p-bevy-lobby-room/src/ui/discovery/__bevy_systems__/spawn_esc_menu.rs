use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use ui::discovery::{
    EscMenu, EscMenuBackdrop, EscMenuFriendsButton, EscMenuLeaveButton, MENU_WIDTH,
};
use ui::discovery::{EscMenuPanel, EscMenuResumeButton, MENU_PANEL_RADIUS};

pub fn spawn_esc_menu(mut commands: Commands) {
    commands.spawn((
        EscMenu,
        GlobalZIndex(ui::discovery::MENU_Z),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        children![
            (
                EscMenuBackdrop,
                Button,
                Hovered::default(),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.0)),
            ),
            (
                EscMenuPanel,
                Node {
                    width: Val::Px(MENU_WIDTH),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(14.0)),
                    border_radius: BorderRadius::all(Val::Px(MENU_PANEL_RADIUS)),
                    ..default()
                },
                BackgroundColor(ui::constants::DRAWER_PANEL_COLOR),
                children![
                    EscMenuPanel::button(EscMenuFriendsButton, "Friends", "Tab"),
                    EscMenuPanel::button(EscMenuLeaveButton, "Leave room", ""),
                    EscMenuPanel::button(EscMenuResumeButton, "Resume", "Esc"),
                ],
            ),
        ],
    ));
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;

    use super::spawn_esc_menu;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, spawn_esc_menu);
        app.update();
        let mut menus = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::EscMenu>>();
        assert_eq!(menus.iter(app.world()).count(), 1);
        let mut friends = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::EscMenuFriendsButton>>();
        assert_eq!(friends.iter(app.world()).count(), 1);
        let mut leaves = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::EscMenuLeaveButton>>();
        assert_eq!(leaves.iter(app.world()).count(), 1);
        let mut resumes = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::EscMenuResumeButton>>();
        assert_eq!(resumes.iter(app.world()).count(), 1);
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(State::new(ui::discovery::Screen::Lobby));
        app.init_resource::<ui::discovery::EscMenuState>();
        app.add_systems(Startup, (spawn_esc_menu, mock_room_chrome).chain());
        app.add_systems(
            Update,
            (
                ui::discovery::bevy_systems::animate_esc_menu,
                ui::discovery::bevy_systems::refresh_menu_leave_button,
                ui::discovery::bevy_systems::animate_hover,
            ),
        );
    }

    fn mock_room_chrome(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(ui::constants::ROOM_COLOR),
            children![(
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(40.0),
                    right: Val::Px(40.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(ui::discovery::PANEL_COLOR),
                children![Text::new("the screen behind the menu")],
            )],
        ));
    }

    fn go_to_lobby(world: &mut World) {
        world.insert_resource(State::new(ui::discovery::Screen::Lobby));
    }

    fn go_to_room(world: &mut World) {
        world.insert_resource(State::new(ui::discovery::Screen::InRoom));
    }

    fn open_menu(world: &mut World) {
        **world.resource_mut::<ui::discovery::EscMenuState>() = true;
    }

    #[test]
    fn spawn_esc_menu_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_esc_menu"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("lobby menu"),
                    apply: |world: &mut World| {
                        go_to_lobby(world);
                        open_menu(world);
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("in-room menu"),
                    apply: |world: &mut World| {
                        go_to_room(world);
                        open_menu(world);
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
