use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use crate::basic::constants::DRAWER_PANEL_COLOR;
use crate::direct_dial;
use direct_dial::{
    DRAWER_BACKDROP_ALPHA, DRAWER_WIDTH, DRAWER_Z, FriendsDrawer, FriendsDrawerBackdrop,
    FriendsDrawerPanel, FriendsDrawerSlide,
};

pub fn spawn_friends_drawer(mut commands: Commands) {
    commands.spawn((
        FriendsDrawer,
        GlobalZIndex(DRAWER_Z),
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        children![
            (
                FriendsDrawerBackdrop,
                Button,
                Hovered::default(),
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, DRAWER_BACKDROP_ALPHA)),
            ),
            (
                FriendsDrawerPanel,
                FriendsDrawerSlide(0.0),
                UiTransform::default(),
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::ZERO,
                    bottom: Val::ZERO,
                    left: Val::ZERO,
                    width: Val::Px(DRAWER_WIDTH),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(10.0),
                    padding: UiRect::all(Val::Px(14.0)),
                    border_radius: BorderRadius {
                        top_left: CornerRadius::ZERO,
                        top_right: CornerRadius::circular(Val::Px(12.0)),
                        bottom_left: CornerRadius::ZERO,
                        bottom_right: CornerRadius::circular(Val::Px(12.0)),
                    },
                    ..default()
                },
                BackgroundColor(DRAWER_PANEL_COLOR),
            ),
        ],
    ));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;

    use super::spawn_friends_drawer;
    use crate::basic::constants::{DRAWER_SLIDE_SECS, ROOM_COLOR};
    use crate::direct_dial;
    use crate::discovery;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, spawn_friends_drawer);
        app.update();
        let mut drawers = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::FriendsDrawer>>();
        assert_eq!(drawers.iter(app.world()).count(), 1);
        let mut backdrops = app
            .world_mut()
            .query_filtered::<(), With<direct_dial::FriendsDrawerBackdrop>>();
        assert_eq!(backdrops.iter(app.world()).count(), 1);
    }

    fn preview_share() -> core_direct_dial::OwnShareAddress {
        core_direct_dial::OwnShareAddress(Some(core_direct_dial::OwnAddress {
            id: identity::PeerId::from("12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC"),
            short: String::from("12D3Koo...99RC"),
            full: Some(String::from(
                "/ip4/127.0.0.1/tcp/4001/p2p/12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC",
            )),
        }))
    }

    fn spawn_mock_screen(mut commands: Commands) {
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
            BackgroundColor(ROOM_COLOR),
            children![(
                Node {
                    position_type: PositionType::Absolute,
                    top: Val::Px(40.0),
                    right: Val::Px(40.0),
                    padding: UiRect::all(Val::Px(12.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    ..default()
                },
                BackgroundColor(discovery::PANEL_COLOR),
                children![Text::new("the screen behind the drawer")],
            )],
        ));
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(preview_share());
        app.init_resource::<direct_dial::FriendsDrawerState>();
        app.add_systems(
            Startup,
            (
                spawn_mock_screen,
                spawn_friends_drawer,
                direct_dial::bevy_systems::spawn_drawer_sections,
            ),
        );
        app.add_systems(
            Update,
            (
                direct_dial::bevy_systems::animate_friends_drawer,
                direct_dial::bevy_systems::refresh_own_address,
            ),
        );
    }

    const fn preview_noop(_world: &mut World) {}

    fn open_fully(world: &mut World) {
        world.insert_resource(direct_dial::FriendsDrawerState(true));
        let mut slides = world.query_filtered::<&mut direct_dial::FriendsDrawerSlide, With<direct_dial::FriendsDrawerPanel>>();
        if let Ok(mut slide) = slides.single_mut(world) {
            **slide = DRAWER_SLIDE_SECS;
        }
    }

    #[test]
    fn spawn_friends_drawer_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_friends_drawer"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("closed"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("open empty"),
                    apply: open_fully,
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
