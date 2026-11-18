use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    INPUT_COLOR, PREVIEW_ROW_GAP, PREVIEW_VIEWPORT_PADDING, RoomPreviewFriends, RoomPreviewViewport,
};

pub fn spawn_preview_friends_card(
    mut commands: Commands,
    viewport: Option<Single<Entity, With<RoomPreviewViewport>>>,
) {
    let Some(viewport) = viewport else {
        return;
    };
    commands.entity(viewport.into_inner()).with_child((
        RoomPreviewFriends,
        Visibility::Hidden,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(PREVIEW_ROW_GAP),
            padding: UiRect::all(Val::Px(PREVIEW_VIEWPORT_PADDING)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(INPUT_COLOR),
    ));
}

#[cfg(test)]
mod tests {
    use super::spawn_preview_friends_card;
    use crate::discovery as core_discovery;
    use crate::identity;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn spawn_preview_friends_card_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("spawn_preview_friends_card"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: |app: &mut App| {
                app.init_resource::<ui::discovery::HoveredOccupancyPill>();
                app.insert_resource(core_discovery::LobbyRows(vec![core_discovery::LobbyRow {
                    name: core_discovery::RoomName::from("alpha"),
                    occupants: 2,
                    room_size: core_discovery::RoomSize::try_from(8).expect("8"),
                    occupancy: core_discovery::RoomOccupancy::HasSpace,
                    friends: 2,
                }]));
                app.insert_resource(ui::discovery::StreamedRoom::default());
                app.insert_resource(ui::discovery::SelectedRoom(Some(
                    core_discovery::RoomName::from("alpha"),
                )));
                app.add_systems(
                    Startup,
                    (
                        ui::discovery::bevy_systems::spawn_room_preview,
                        spawn_preview_friends_card,
                    )
                        .chain(),
                );
                app.add_systems(
                    Update,
                    (
                        ui::discovery::bevy_systems::refresh_room_preview,
                        ui::discovery::bevy_systems::rebuild_preview_friends,
                    ),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("preview only"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("friends shown"),
                    apply: |world: &mut World| {
                        **world.resource_mut::<ui::discovery::HoveredOccupancyPill>() =
                            Some(core_discovery::RoomName::from("alpha"));
                        world.insert_resource(core_discovery::RoomFriendNames(
                            std::collections::BTreeMap::from([(
                                core_discovery::RoomName::from("alpha"),
                                vec![
                                    identity::Username::from("alice"),
                                    identity::Username::from("bob"),
                                ],
                            )]),
                        ));
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
