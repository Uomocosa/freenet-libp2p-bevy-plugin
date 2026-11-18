use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;

use ui::discovery::{RoomSearchIndicator, RoomSearchTooltip};

pub fn show_room_search_tooltip(
    indicators: Query<&Hovered, With<RoomSearchIndicator>>,
    tooltip: Single<&mut Visibility, With<RoomSearchTooltip>>,
) {
    let is_hovered = indicators.iter().any(Hovered::get);
    let wanted = if is_hovered {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let mut tooltip = tooltip.into_inner();
    if *tooltip != wanted {
        *tooltip = wanted;
    }
}

#[cfg(test)]
mod tests {
    use super::show_room_search_tooltip;
    use crate::ui;
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;

    #[test]
    fn show_room_search_tooltip_ui_png_preview() {
        #[derive(Component)]
        struct PinSearchHover;

        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("show_room_search_tooltip"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    commands.entity(root).with_children(|root| {
                        ui::discovery::bevy_systems::spawn_rooms_header(&mut root.reborrow());
                    });
                });
                app.add_systems(
                    PostStartup,
                    lele_bevy_preview::scene::bevy_systems::center_roots,
                );
                app.add_systems(
                    Update,
                    (
                        |mut commands: Commands, pinned: Query<Entity, With<PinSearchHover>>| {
                            for entity in &pinned {
                                commands.entity(entity).insert(Hovered(true));
                            }
                        },
                        show_room_search_tooltip,
                    )
                        .chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("not hovered"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("hovered"),
                    apply: |world: &mut World| {
                        let mut indicators = world
                            .query_filtered::<Entity, With<ui::discovery::RoomSearchIndicator>>();
                        let indicators: Vec<Entity> = indicators.iter(world).collect();
                        for entity in indicators {
                            world
                                .entity_mut(entity)
                                .insert((Hovered(true), PinSearchHover));
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
