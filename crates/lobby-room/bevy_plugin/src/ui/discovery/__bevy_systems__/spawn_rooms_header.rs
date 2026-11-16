use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;

use ui::discovery::{
    ACCENT_COLOR, CONNECTING_TEXT_COLOR, ROOM_SEARCH_LABEL_FONT_SIZE, ROOM_SEARCH_SPINNER_SIZE,
    ROOM_SEARCH_TOOLTIP_BG_COLOR, ROOM_SEARCH_TOOLTIP_FONT_SIZE, ROOM_SEARCH_TOOLTIP_Z,
    RoomSearchIndicator, RoomSearchTooltip, RoomsHeader, SPINNER_TRACK_COLOR, SpinnerRing,
};

pub fn spawn_rooms_header(root: &mut ChildSpawnerCommands) {
    root.spawn((
        RoomsHeader,
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(6.0),
            ..default()
        },
    ))
    .with_children(|header| {
        header.spawn((
            Text::new("Rooms"),
            TextFont {
                font_size: FontSize::Px(ROOM_SEARCH_LABEL_FONT_SIZE),
                ..default()
            },
            TextColor(CONNECTING_TEXT_COLOR),
        ));
        header
            .spawn((
                RoomSearchIndicator,
                Hovered::default(),
                Node {
                    width: Val::Px(ROOM_SEARCH_SPINNER_SIZE),
                    height: Val::Px(ROOM_SEARCH_SPINNER_SIZE),
                    ..default()
                },
            ))
            .with_children(|indicator| {
                indicator.spawn((
                    SpinnerRing,
                    Node {
                        width: Val::Px(ROOM_SEARCH_SPINNER_SIZE),
                        height: Val::Px(ROOM_SEARCH_SPINNER_SIZE),
                        border: UiRect::all(Val::Px(2.0)),
                        border_radius: BorderRadius::MAX,
                        ..default()
                    },
                    BorderColor {
                        top: ACCENT_COLOR,
                        ..BorderColor::all(SPINNER_TRACK_COLOR)
                    },
                    UiTransform::default(),
                ));
                indicator.spawn((
                    RoomSearchTooltip,
                    Visibility::Hidden,
                    GlobalZIndex(ROOM_SEARCH_TOOLTIP_Z),
                    Node {
                        position_type: PositionType::Absolute,
                        top: Val::Px(18.0),
                        left: Val::Px(0.0),
                        padding: UiRect::axes(Val::Px(6.0), Val::Px(3.0)),
                        border_radius: BorderRadius::all(Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(ROOM_SEARCH_TOOLTIP_BG_COLOR),
                    Text::new("Searching for more rooms..."),
                    TextFont {
                        font_size: FontSize::Px(ROOM_SEARCH_TOOLTIP_FONT_SIZE),
                        ..default()
                    },
                    TextColor(CONNECTING_TEXT_COLOR),
                    TextLayout::no_wrap(),
                ));
            });
    });
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;

    use super::spawn_rooms_header;

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_systems(Startup, |mut commands: Commands| {
            let root = commands.spawn_empty().id();
            commands
                .entity(root)
                .with_children(|root| spawn_rooms_header(&mut root.reborrow()));
        });
        app.update();
        let mut headers = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::RoomsHeader>>();
        assert_eq!(headers.iter(app.world()).count(), 1);
        let mut rings = app
            .world_mut()
            .query_filtered::<(), With<ui::discovery::SpinnerRing>>();
        assert_eq!(rings.iter(app.world()).count(), 1);
        let mut indicators = app
            .world_mut()
            .query_filtered::<Entity, With<ui::discovery::RoomSearchIndicator>>();
        let indicator = indicators.single(app.world()).expect("search indicator");
        let mut tooltips = app
            .world_mut()
            .query_filtered::<Entity, With<ui::discovery::RoomSearchTooltip>>();
        let tooltip = tooltips.single(app.world()).expect("search tooltip");
        let tooltip_visibility = *app
            .world()
            .get::<Visibility>(tooltip)
            .expect("tooltip visibility");
        assert_eq!(tooltip_visibility, Visibility::Hidden);
        let GlobalZIndex(tooltip_z) = *app.world().get::<GlobalZIndex>(tooltip).expect("tooltip z");
        assert_eq!(tooltip_z, ui::discovery::ROOM_SEARCH_TOOLTIP_Z);
        assert_eq!(
            app.world()
                .entity(indicator)
                .get::<Children>()
                .expect("indicator children")
                .len(),
            2
        );
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
        commands
            .entity(root)
            .with_children(|root| spawn_rooms_header(&mut root.reborrow()));
    }

    fn preview_build(app: &mut App) {
        app.add_systems(Startup, spawn_header_preview);
        app.add_systems(
            PostStartup,
            lele_bevy_preview::scene::bevy_systems::center_roots,
        );
        app.add_systems(
            Update,
            (
                pin_search_hover,
                ui::discovery::bevy_systems::show_room_search_tooltip,
            )
                .chain(),
        );
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn spawn_rooms_header_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_rooms_header"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("idle"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("searching hovered"),
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
