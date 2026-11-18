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
    use super::spawn_rooms_header;
    use crate::ui;
    use bevy::picking::hover::Hovered;
    use bevy::prelude::*;

    #[test]
    fn spawn_rooms_header_ui_png_preview() {
        #[derive(Component)]
        struct PinSearchHover;

        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_rooms_header"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.add_systems(Startup, |mut commands: Commands| {
                    let root = commands
                        .spawn(lele_bevy_preview::scene::Scene::centered_root())
                        .id();
                    commands
                        .entity(root)
                        .with_children(|root| spawn_rooms_header(&mut root.reborrow()));
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
                        ui::discovery::bevy_systems::show_room_search_tooltip,
                    )
                        .chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("idle"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("searching hovered"),
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
