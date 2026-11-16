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

    fn app_with() -> (App, Entity, Entity) {
        let mut app = App::new();
        let indicator = app
            .world_mut()
            .spawn((ui::discovery::RoomSearchIndicator, Hovered::default()))
            .id();
        let tooltip = app
            .world_mut()
            .spawn((ui::discovery::RoomSearchTooltip, Visibility::Hidden))
            .id();
        app.add_systems(Update, show_room_search_tooltip);
        (app, indicator, tooltip)
    }

    fn tooltip_visibility(app: &App, tooltip: Entity) -> Visibility {
        *app.world()
            .get::<Visibility>(tooltip)
            .expect("search tooltip")
    }

    #[test]
    fn test_usage() {
        let (mut app, _indicator, tooltip) = app_with();
        app.update();
        assert_eq!(tooltip_visibility(&app, tooltip), Visibility::Hidden);
    }

    #[test]
    fn hovering_the_search_indicator_shows_the_searching_tooltip() {
        let (mut app, indicator, tooltip) = app_with();
        app.world_mut().entity_mut(indicator).insert(Hovered(true));
        app.update();
        assert_eq!(tooltip_visibility(&app, tooltip), Visibility::Visible);
    }

    #[test]
    fn leaving_the_search_indicator_hides_the_searching_tooltip() {
        let (mut app, indicator, tooltip) = app_with();
        app.world_mut().entity_mut(indicator).insert(Hovered(true));
        app.update();
        app.world_mut().entity_mut(indicator).insert(Hovered(false));
        app.update();
        assert_eq!(tooltip_visibility(&app, tooltip), Visibility::Hidden);
    }

    #[derive(Component)]
    struct PinSearchHover;

    fn pin_search_hover(mut commands: Commands, pinned: Query<Entity, With<PinSearchHover>>) {
        for entity in &pinned {
            commands.entity(entity).insert(Hovered(true));
        }
    }

    fn hover_indicator(world: &mut World) {
        let mut indicators =
            world.query_filtered::<Entity, With<ui::discovery::RoomSearchIndicator>>();
        let indicators: Vec<Entity> = indicators.iter(world).collect();
        for entity in indicators {
            world
                .entity_mut(entity)
                .insert((Hovered(true), PinSearchHover));
        }
    }

    fn spawn_header_preview(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.entity(root).with_children(|root| {
            ui::discovery::bevy_systems::spawn_rooms_header(&mut root.reborrow());
        });
    }

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, spawn_header_preview);
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
        app.add_systems(Update, (pin_search_hover, show_room_search_tooltip).chain());
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn show_room_search_tooltip_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("show_room_search_tooltip"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("not hovered"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("hovered"),
                    apply: hover_indicator,
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
