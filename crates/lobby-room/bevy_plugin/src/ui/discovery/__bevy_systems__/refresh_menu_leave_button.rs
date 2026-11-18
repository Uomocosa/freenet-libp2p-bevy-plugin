use crate::ui;
use bevy::prelude::*;

use ui::discovery::{EscMenuLeaveButton, Screen};

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
    use crate::ui;
    use bevy::prelude::*;

    use super::refresh_menu_leave_button;
    use ui::discovery::Screen;

    #[test]
    fn refresh_menu_leave_button_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_menu_leave_button"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(State::new(Screen::Lobby));
                app.init_resource::<ui::discovery::EscMenuState>();
                app.add_systems(Startup, |mut commands: Commands| {
                    ui::discovery::bevy_systems::spawn_esc_menu(commands.reborrow());
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
                        children![Text::new("the screen behind the menu")],
                    ));
                });
                app.add_systems(
                    Update,
                    (
                        ui::discovery::bevy_systems::animate_esc_menu,
                        refresh_menu_leave_button,
                        ui::discovery::bevy_systems::animate_hover,
                    ),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("lobby: no leave button"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<ui::discovery::EscMenuState>() = true;
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("in room: leave button shown"),
                    apply: |world: &mut World| {
                        world.insert_resource(State::new(Screen::InRoom));
                        **world.resource_mut::<ui::discovery::EscMenuState>() = true;
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
