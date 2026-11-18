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

    fn row(friends: usize) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from("alpha"),
            occupants: 3,
            room_size: core_discovery::RoomSize::try_from(4).expect("4"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends,
        }
    }

    fn app_with(friends: usize) -> App {
        let mut app = App::new();
        app.init_resource::<ui::discovery::HoveredOccupancyPill>();
        app.add_systems(Startup, move |mut commands: Commands| {
            let list = commands.spawn_empty().id();
            commands.entity(list).with_children(|rooms| {
                ui::discovery::bevy_systems::spawn_room_row(&mut rooms.reborrow(), &row(friends));
            });
        });
        app.add_systems(Update, show_room_bubble_tooltip);
        app
    }

    fn first_with<C: Component>(app: &mut App) -> Entity {
        let mut query = app.world_mut().query_filtered::<Entity, With<C>>();
        query.single(app.world()).expect("marked entity")
    }

    fn visibility(app: &App, entity: Entity) -> Visibility {
        *app.world().get::<Visibility>(entity).expect("visibility")
    }

    fn tooltip_text(app: &App, tooltip: Entity) -> String {
        app.world()
            .get::<Text>(tooltip)
            .expect("tooltip text")
            .as_str()
            .to_string()
    }

    fn hover_pill(app: &mut App) {
        let pill = first_with::<ui::discovery::RoomBubble>(app);
        app.world_mut().entity_mut(pill).insert(Hovered(true));
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(1);
        app.update();
        let tooltip = first_with::<ui::discovery::RoomBubbleTooltip>(&mut app);
        assert_eq!(visibility(&app, tooltip), Visibility::Hidden);
        hover_pill(&mut app);
        app.update();
        assert_eq!(visibility(&app, tooltip), Visibility::Visible);
    }

    #[test]
    fn the_tooltip_stays_hidden_by_default() {
        let mut app = app_with(0);
        app.update();
        let tooltip = first_with::<ui::discovery::RoomBubbleTooltip>(&mut app);
        assert_eq!(visibility(&app, tooltip), Visibility::Hidden);
        assert_eq!(
            **app
                .world()
                .resource::<ui::discovery::HoveredOccupancyPill>(),
            None
        );
    }

    #[test]
    fn hovering_the_pill_shows_the_friends_tooltip_and_publishes_its_room() {
        let mut app = app_with(1);
        app.update();
        let tooltip = first_with::<ui::discovery::RoomBubbleTooltip>(&mut app);
        hover_pill(&mut app);
        app.update();
        assert_eq!(visibility(&app, tooltip), Visibility::Visible);
        assert_eq!(tooltip_text(&app, tooltip), "3 of 4 players | 1 friend");
        assert_eq!(
            **app
                .world()
                .resource::<ui::discovery::HoveredOccupancyPill>(),
            Some(core_discovery::RoomName::from("alpha"))
        );
    }

    #[test]
    fn leaving_the_pill_hides_the_tooltip_and_clears_its_room() {
        let mut app = app_with(2);
        app.update();
        let tooltip = first_with::<ui::discovery::RoomBubbleTooltip>(&mut app);
        hover_pill(&mut app);
        app.update();
        assert_eq!(visibility(&app, tooltip), Visibility::Visible);
        assert_eq!(tooltip_text(&app, tooltip), "3 of 4 players | 2 friends");
        let pill = first_with::<ui::discovery::RoomBubble>(&mut app);
        app.world_mut().entity_mut(pill).insert(Hovered(false));
        app.update();
        assert_eq!(visibility(&app, tooltip), Visibility::Hidden);
        assert_eq!(
            **app
                .world()
                .resource::<ui::discovery::HoveredOccupancyPill>(),
            None
        );
    }

    fn preview_row(name: &str, occupants: usize, friends: usize) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends,
        }
    }

    fn spawn_preview_rows(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        let list = commands
            .spawn((
                ChildOf(root),
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
            ))
            .id();
        commands.entity(list).with_children(|rooms| {
            for (name, occupants, friends) in [("alpha", 2, 1), ("beta", 0, 0), ("gamma", 4, 2)] {
                ui::discovery::bevy_systems::spawn_room_row(
                    &mut rooms.reborrow(),
                    &preview_row(name, occupants, friends),
                );
            }
        });
    }

    #[derive(Component)]
    struct PinPillHover;

    fn pin_pill_hover(mut commands: Commands, pinned: Query<Entity, With<PinPillHover>>) {
        for entity in &pinned {
            commands.entity(entity).insert(Hovered(true));
        }
    }

    fn hover_alpha_pill(world: &mut World) {
        let mut rows = world.query::<(Entity, &ui::discovery::RoomButton)>();
        let Some(row) = rows
            .iter(world)
            .find(|(_entity, button)| button.as_str() == "alpha")
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
            world.entity_mut(pill).insert(PinPillHover);
        }
    }

    fn preview_build(app: &mut App) {
        app.init_resource::<ui::discovery::HoveredOccupancyPill>();
        app.add_systems(Startup, spawn_preview_rows);
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
        app.add_systems(Update, (pin_pill_hover, show_room_bubble_tooltip).chain());
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn show_room_bubble_tooltip_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("show_room_bubble_tooltip"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("pill not hovered"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("occupancy pill hovered"),
                    apply: hover_alpha_pill,
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
