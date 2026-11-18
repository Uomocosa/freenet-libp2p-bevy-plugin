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
    use bevy::ui::Pressed;

    use super::finish_friends_beat;
    use ui::discovery::{EscMenuFriendsButton, EscMenuState};

    #[test]
    fn finish_friends_beat_ui_mp4_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("finish_friends_beat"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(freenet_libp2p_bevy_core::direct_dial::OwnShareAddress(
                    Some(freenet_libp2p_bevy_core::direct_dial::OwnAddress {
                        id: freenet_libp2p_bevy_core::identity::PeerId::from(
                            "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC",
                        ),
                        short: String::from("12D3Koo...99RC"),
                        full: None,
                    }),
                ));
                app.init_resource::<EscMenuState>();
                app.init_resource::<ui::discovery::FriendsPressBeat>();
                app.init_resource::<ui::direct_dial::FriendsDrawerState>();
                app.add_systems(
                    Startup,
                    (
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
                            ui::discovery::bevy_systems::spawn_esc_menu(commands.reborrow());
                            ui::direct_dial::bevy_systems::spawn_friends_drawer(commands);
                        },
                        ui::direct_dial::bevy_systems::spawn_drawer_sections,
                    )
                        .chain(),
                );
                app.add_systems(
                    Update,
                    (
                        ui::discovery::bevy_systems::animate_esc_menu,
                        ui::discovery::bevy_systems::animate_hover,
                        ui::discovery::bevy_systems::start_friends_beat,
                        finish_friends_beat,
                        ui::direct_dial::bevy_systems::refresh_own_address,
                        ui::direct_dial::bevy_systems::animate_friends_drawer,
                    ),
                );
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("menu open"),
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
            }],
            timeline: Some(lele_bevy_preview::scene::Timeline {
                label: String::from("pressed_beat_drawer_in"),
                frames: 26,
                fps: 30,
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
            }),
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
