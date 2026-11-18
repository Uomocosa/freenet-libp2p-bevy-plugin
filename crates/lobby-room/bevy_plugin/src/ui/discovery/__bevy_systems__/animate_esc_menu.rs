use crate::ui;
use bevy::prelude::*;
use bevy::ui::Pressed;

use ui::discovery::{
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
    use crate::ui;
    use bevy::prelude::*;
    use bevy::ui::Pressed;

    use super::animate_esc_menu;
    use ui::discovery::{EscMenuFriendsButton, EscMenuState};

    #[test]
    fn animate_esc_menu_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("animate_esc_menu"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.init_resource::<EscMenuState>();
                app.add_systems(
                    Startup,
                    (
                        |mut commands: Commands| {
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
                                BackgroundColor(Color::srgb(0.16, 0.18, 0.22)),
                                children![Text::new("the screen behind the menu")],
                            ));
                        },
                        |mut commands: Commands| {
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
                        },
                    )
                        .chain(),
                );
                app.add_systems(
                    Update,
                    (animate_esc_menu, ui::discovery::bevy_systems::animate_hover).chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("closed"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("open"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<EscMenuState>() = true;
                    },
                },
                lele_bevy_preview::scene::State {
                    label: String::from("friends pressed"),
                    apply: |world: &mut World| {
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
