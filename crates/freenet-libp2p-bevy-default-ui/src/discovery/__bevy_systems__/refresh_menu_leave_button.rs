use bevy::prelude::*;

use crate::discovery;
use discovery::{EscMenuLeaveButton, Screen};

pub fn refresh_menu_leave_button(
    screen: Option<Res<State<Screen>>>,
    mut button: Single<&mut Visibility, With<EscMenuLeaveButton>>,
) {
    let is_in_room = screen.is_some_and(|screen| **screen == Screen::InRoom);
    let wanted = if is_in_room {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if **button != wanted {
        **button = wanted;
    }
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use bevy::state::app::StatesPlugin;

    use super::refresh_menu_leave_button;
    use crate::basic::constants::ROOM_COLOR;
    use crate::discovery;
    use crate::discovery::bevy_systems::spawn_esc_menu;
    use discovery::{EscMenuLeaveButton, Screen};

    fn app_with(screen: Screen) -> (App, Entity) {
        let mut app = App::new();
        app.add_plugins(StatesPlugin);
        app.init_state::<discovery::Screen>();
        app.world_mut()
            .resource_mut::<NextState<Screen>>()
            .set(screen);
        app.world_mut().run_schedule(StateTransition);
        let button = app
            .world_mut()
            .spawn((EscMenuLeaveButton, Visibility::Inherited))
            .id();
        app.add_systems(Update, refresh_menu_leave_button);
        (app, button)
    }

    fn visibility_of(app: &App, entity: Entity) -> Visibility {
        *app.world().get::<Visibility>(entity).expect("button")
    }

    #[test]
    fn test_usage() {
        let (mut app, button) = app_with(Screen::Lobby);
        app.update();
        assert_eq!(visibility_of(&app, button), Visibility::Hidden);
    }

    #[test]
    fn the_leave_button_hides_outside_a_room() {
        for screen in [
            Screen::Connecting,
            Screen::Lobby,
            Screen::JoiningRoom,
            Screen::JoinFailed,
        ] {
            let (mut app, button) = app_with(screen);
            app.update();
            assert_eq!(visibility_of(&app, button), Visibility::Hidden);
        }
    }

    #[test]
    fn the_leave_button_shows_in_a_room() {
        let (mut app, button) = app_with(Screen::InRoom);
        app.update();
        assert_eq!(visibility_of(&app, button), Visibility::Inherited);
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
            BackgroundColor(ROOM_COLOR),
            children![Text::new("the screen behind the menu")],
        ));
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(State::new(discovery::Screen::Lobby));
        app.init_resource::<discovery::EscMenuState>();
        app.add_systems(Startup, spawn_menu);
        app.add_systems(
            Update,
            (
                discovery::bevy_systems::animate_esc_menu,
                refresh_menu_leave_button,
                discovery::bevy_systems::animate_hover,
            ),
        );
    }

    fn open_menu(world: &mut World) {
        **world.resource_mut::<discovery::EscMenuState>() = true;
    }

    fn go_to_room(world: &mut World) {
        world.insert_resource(State::new(Screen::InRoom));
        open_menu(world);
    }

    #[test]
    fn refresh_menu_leave_button_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_menu_leave_button"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("lobby: no leave button"),
                    apply: open_menu,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("in room: leave button shown"),
                    apply: go_to_room,
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
