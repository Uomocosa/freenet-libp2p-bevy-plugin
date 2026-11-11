use bevy::prelude::*;
use bevy::ui::Pressed;

use crate::discovery;
use discovery::{
    EscMenu, EscMenuBackdrop, EscMenuFriendsButton, EscMenuState, MENU_BACKDROP_ALPHA,
};

pub fn animate_esc_menu(
    state: Res<EscMenuState>,
    root: Single<&mut Visibility, With<EscMenu>>,
    backdrop: Single<&mut BackgroundColor, With<EscMenuBackdrop>>,
    pressed: Query<Entity, (With<EscMenuFriendsButton>, With<Pressed>)>,
    mut commands: Commands,
) {
    let state = state.into_inner();
    let wanted_visibility = if **state {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let mut root = root.into_inner();
    if *root != wanted_visibility {
        *root = wanted_visibility;
    }
    let mut backdrop = backdrop.into_inner();
    let wanted_backdrop = if **state {
        Color::srgba(0.0, 0.0, 0.0, MENU_BACKDROP_ALPHA)
    } else {
        Color::srgba(0.0, 0.0, 0.0, 0.0)
    };
    let BackgroundColor(current) = &mut *backdrop;
    if *current != wanted_backdrop {
        *current = wanted_backdrop;
    }
    if !**state {
        for entity in &pressed {
            commands.entity(entity).remove::<Pressed>();
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::animate_esc_menu;
    use crate::discovery;
    use crate::discovery::bevy_systems::spawn_esc_menu;
    use discovery::{EscMenu, EscMenuBackdrop, EscMenuFriendsButton, EscMenuState};

    fn app_with(open: bool) -> (App, Entity, Entity, Entity) {
        let mut app = App::new();
        app.insert_resource(EscMenuState(open));
        let root = app.world_mut().spawn((EscMenu, Visibility::Hidden)).id();
        let backdrop = app
            .world_mut()
            .spawn((ChildOf(root), EscMenuBackdrop, BackgroundColor::default()))
            .id();
        let friends = app
            .world_mut()
            .spawn((ChildOf(root), EscMenuFriendsButton))
            .id();
        app.add_systems(Update, animate_esc_menu);
        (app, root, backdrop, friends)
    }

    #[test]
    fn test_usage() {
        let (mut app, root, _backdrop, _friends) = app_with(false);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(root).expect("root"),
            Visibility::Hidden
        );
    }

    #[test]
    fn an_open_menu_shows_the_root_and_dimmed_backdrop() {
        let (mut app, root, backdrop, _friends) = app_with(true);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(root).expect("root"),
            Visibility::Visible
        );
        let BackgroundColor(color) = *app.world().get::<BackgroundColor>(backdrop).expect("bg");
        assert!(color.alpha() > 0.0);
    }

    #[test]
    fn a_closed_menu_hides_the_root_and_clears_a_stale_press() {
        let (mut app, root, _backdrop, friends) = app_with(false);
        app.world_mut().entity_mut(friends).insert(Pressed);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(root).expect("root"),
            Visibility::Hidden
        );
        assert!(
            app.world().get::<Pressed>(friends).is_none(),
            "a stale press never lingers into the next open menu"
        );
    }

    #[test]
    fn an_open_menu_keeps_the_friends_press() {
        let (mut app, _root, _backdrop, friends) = app_with(true);
        app.world_mut().entity_mut(friends).insert(Pressed);
        app.update();
        assert!(app.world().get::<Pressed>(friends).is_some());
    }

    fn spawn_menu(mut commands: Commands) {
        spawn_esc_menu(commands.reborrow());
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
    }

    fn preview_build(app: &mut App) {
        app.init_resource::<EscMenuState>();
        app.add_systems(Startup, (spawn_menu, mock_screen).chain());
        app.add_systems(
            Update,
            (animate_esc_menu, discovery::bevy_systems::animate_hover).chain(),
        );
    }

    fn mock_screen(mut commands: Commands) {
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
    }

    const fn preview_noop(_world: &mut World) {}

    fn open_menu(world: &mut World) {
        **world.resource_mut::<EscMenuState>() = true;
    }

    fn press_friends(world: &mut World) {
        open_menu(world);
        let mut friends = world.query_filtered::<Entity, With<EscMenuFriendsButton>>();
        let pressed: Vec<Entity> = friends.iter(world).collect();
        for entity in pressed {
            world.entity_mut(entity).insert((
                Pressed,
                bevy::picking::hover::Hovered(true),
                discovery::HoverGlow(1.0),
            ));
        }
    }

    #[test]
    fn animate_esc_menu_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("animate_esc_menu"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("closed"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("open"),
                    apply: open_menu,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("friends pressed"),
                    apply: press_friends,
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
