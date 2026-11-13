use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{EscMenuFriendsButton, EscMenuState, FriendsPressBeat};

pub fn finish_friends_beat(
    time: Res<Time>,
    mut beat: ResMut<FriendsPressBeat>,
    mut menu: ResMut<EscMenuState>,
    mut drawer: Option<ResMut<ui::direct_dial::FriendsDrawerState>>,
    pressed: Query<Entity, (With<EscMenuFriendsButton>, With<Pressed>)>,
    mut commands: Commands,
) {
    if **beat <= 0.0 {
        return;
    }
    let remaining = **beat - time.into_inner().delta_secs();
    if remaining > 0.0 {
        **beat = remaining;
        return;
    }
    **beat = 0.0;
    for entity in &pressed {
        commands.entity(entity).remove::<Pressed>();
    }
    if !**menu {
        return;
    }
    menu.close();
    if let Some(drawer) = drawer.as_deref_mut()
        && !**drawer
    {
        drawer.toggle();
    }
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::prelude::*;
    use bevy::time::TimeUpdateStrategy;
    use bevy::ui::Pressed;

    use super::super::animate_esc_menu::animate_esc_menu;
    use super::super::animate_hover::animate_hover;
    use super::super::spawn_esc_menu::spawn_esc_menu;
    use super::super::start_friends_beat::start_friends_beat;
    use super::finish_friends_beat;
    use ui::direct_dial::bevy_systems::{
        animate_friends_drawer, refresh_own_address, spawn_drawer_sections, spawn_friends_drawer,
    };
    use ui::discovery::{EscMenuFriendsButton, EscMenuState, FriendsPressBeat};

    fn app_with(drawer_open: bool) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(TimeUpdateStrategy::ManualDuration(
            std::time::Duration::from_millis(50),
        ));
        app.insert_resource(EscMenuState(true));
        app.insert_resource(FriendsPressBeat(ui::discovery::MENU_FRIENDS_BEAT_SECS));
        app.insert_resource(ui::direct_dial::FriendsDrawerState(drawer_open));
        let button = app.world_mut().spawn(EscMenuFriendsButton).id();
        app.add_systems(Update, finish_friends_beat);
        (app, button)
    }

    #[test]
    fn test_usage() {
        let (mut app, button) = app_with(false);
        app.world_mut().entity_mut(button).insert(Pressed);
        for _ in 0..5 {
            app.update();
        }
        assert!(!**app.world().resource::<EscMenuState>());
        assert!(
            **app
                .world()
                .resource::<ui::direct_dial::FriendsDrawerState>()
        );
        assert!(
            app.world().get::<Pressed>(button).is_none(),
            "the press does not linger once the beat has fired"
        );
    }

    #[test]
    fn an_idle_beat_never_fires() {
        let (mut app, _button) = app_with(false);
        **app.world_mut().resource_mut::<FriendsPressBeat>() = 0.0;
        app.update();
        assert!(**app.world().resource::<EscMenuState>());
    }

    #[test]
    fn mid_beat_the_menu_is_still_open() {
        let (mut app, _button) = app_with(false);
        app.update();
        assert!(**app.world().resource::<EscMenuState>());
        assert!(**app.world().resource::<FriendsPressBeat>() > 0.0);
    }

    #[test]
    fn a_fired_beat_never_closes_an_already_open_drawer() {
        let (mut app, _button) = app_with(true);
        for _ in 0..5 {
            app.update();
        }
        assert!(
            **app
                .world()
                .resource::<ui::direct_dial::FriendsDrawerState>()
        );
        assert!(!**app.world().resource::<EscMenuState>());
    }

    #[test]
    fn a_fired_beat_leaves_a_closed_menu_closed() {
        let (mut app, button) = app_with(false);
        **app.world_mut().resource_mut::<EscMenuState>() = false;
        for _ in 0..5 {
            app.update();
        }
        assert!(!**app.world().resource::<EscMenuState>());
        assert!(
            !**app
                .world()
                .resource::<ui::direct_dial::FriendsDrawerState>()
        );
        assert!(app.world().get::<Pressed>(button).is_none());
    }

    fn preview_share() -> freenet_libp2p_bevy_core::direct_dial::OwnShareAddress {
        freenet_libp2p_bevy_core::direct_dial::OwnShareAddress(Some(
            freenet_libp2p_bevy_core::direct_dial::OwnAddress {
                id: freenet_libp2p_bevy_core::identity::PeerId::from(
                    "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC",
                ),
                short: String::from("12D3Koo...99RC"),
                full: None,
            },
        ))
    }

    fn spawn_menu_chrome(mut commands: Commands) {
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
            BackgroundColor(Color::srgb(0.16, 0.18, 0.22)),
            children![Text::new("the screen behind the menu")],
        ));
        spawn_esc_menu(commands.reborrow());
        spawn_friends_drawer(commands);
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(preview_share());
        app.init_resource::<EscMenuState>();
        app.init_resource::<ui::discovery::FriendsPressBeat>();
        app.init_resource::<ui::direct_dial::FriendsDrawerState>();
        app.add_systems(Startup, (spawn_menu_chrome, spawn_drawer_sections).chain());
        app.add_systems(
            Update,
            (
                animate_esc_menu,
                animate_hover,
                start_friends_beat,
                finish_friends_beat,
                refresh_own_address,
                animate_friends_drawer,
            ),
        );
    }

    fn press_friends(world: &mut World) {
        **world.resource_mut::<EscMenuState>() = true;
        let friends: Vec<Entity> = world
            .query_filtered::<Entity, With<EscMenuFriendsButton>>()
            .iter(world)
            .collect();
        for entity in friends {
            world.entity_mut(entity).insert((
                Pressed,
                bevy::picking::hover::Hovered(true),
                ui::discovery::HoverGlow(1.0),
            ));
        }
    }

    #[test]
    fn finish_friends_beat_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("finish_friends_beat"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("menu open"),
                apply: press_friends,
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("pressed_beat_drawer_in"),
                frames: 26,
                fps: 30,
                apply: press_friends,
            }),
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
