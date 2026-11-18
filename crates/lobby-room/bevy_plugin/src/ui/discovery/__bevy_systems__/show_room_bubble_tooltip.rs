use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;

use ui::discovery::{HoveredOccupancyPill, RoomBubble, RoomBubbleTooltip, RoomButton};

pub fn show_room_bubble_tooltip(
    mut hovered_pill: ResMut<HoveredOccupancyPill>,
    pills: Query<(&Hovered, &ChildOf), With<RoomBubble>>,
    rows: Query<&RoomButton>,
    mut tooltips: Query<(&ChildOf, &mut Visibility), With<RoomBubbleTooltip>>,
) {
    let mut hovered_room = None;
    for (child_of, mut visibility) in &mut tooltips {
        let Ok((hovered, pill_row)) = pills.get(child_of.parent()) else {
            if *visibility != Visibility::Hidden {
                *visibility = Visibility::Hidden;
            }
            continue;
        };
        let is_hovered = hovered.get();
        let wanted = if is_hovered {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
        if *visibility != wanted {
            *visibility = wanted;
        }
        if is_hovered {
            hovered_room = rows
                .get(pill_row.parent())
                .ok()
                .map(|button| (**button).clone());
        }
    }
    if **hovered_pill != hovered_room {
        **hovered_pill = hovered_room;
    }
}

#[cfg(test)]
mod tests {
    use super::show_room_bubble_tooltip;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_core::ui as core_ui;

    #[test]
    fn show_room_bubble_tooltip_ui_png_preview() {
        fn hover_pill_of(world: &mut World, room: &str) {
            let mut rows = world.query::<(Entity, &ui::discovery::RoomButton)>();
            let Some(row) = rows
                .iter(world)
                .find(|(_entity, button)| button.as_str() == room)
                .map(|(entity, _button)| entity)
            else {
                return;
            };
            let mut children = world.query::<(Entity, &ChildOf, &ui::discovery::RoomBubble)>();
            let pills: Vec<Entity> = children
                .iter(world)
                .filter(|(_entity, link, _bubble)| link.parent() == row)
                .map(|(entity, _link, _bubble)| entity)
                .collect();
            for pill in pills {
                world.entity_mut(pill).insert(Hovered(true));
            }
        }

        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");

        lele_bevy_preview::scene::Scene {
            name: String::from("show_room_bubble_tooltip"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_plugins(core_ui::floating::UiPlugin);
                app.init_resource::<ui::discovery::HoveredOccupancyPill>();
                app.add_systems(Startup, |mut commands: Commands| {
                    let preview_row =
                        |name: &str, occupants: usize, friends: usize| core_discovery::LobbyRow {
                            name: core_discovery::RoomName::from(name),
                            occupants,
                            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
                            occupancy: core_discovery::RoomOccupancy::HasSpace,
                            friends,
                        };
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    let list = commands
                        .spawn((
                            ChildOf(root),
                            Node {
                                position_type: PositionType::Absolute,
                                top: Val::Px(48.0),
                                right: Val::Px(ui::discovery::LOBBY_PANEL_RIGHT),
                                width: Val::Px(ui::discovery::LOBBY_PANEL_WIDTH),
                                flex_direction: FlexDirection::Column,
                                row_gap: Val::Px(4.0),
                                padding: UiRect::all(Val::Px(8.0)),
                                ..default()
                            },
                        ))
                        .id();
                    commands.entity(list).with_children(|rooms| {
                        for (name, occupants, friends) in
                            [("alpha", 2, 1), ("beta", 0, 0), ("gamma", 4, 2)]
                        {
                            ui::discovery::bevy_systems::spawn_room_row(
                                &mut rooms.reborrow(),
                                &preview_row(name, occupants, friends),
                            );
                        }
                    });
                });
                app.add_systems(
                    PostStartup,
                    lele_bevy_preview::scene::bevy_systems::center_roots,
                );
                app.add_systems(Update, show_room_bubble_tooltip);
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("pill not hovered"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("occupancy pill hovered"),
                    apply: |world: &mut World| hover_pill_of(world, "alpha"),
                },
                lele_bevy_preview::scene::State {
                    label: String::from("occupancy pill hovered at the bottom right edge"),
                    apply: |world: &mut World| hover_pill_of(world, "gamma"),
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
