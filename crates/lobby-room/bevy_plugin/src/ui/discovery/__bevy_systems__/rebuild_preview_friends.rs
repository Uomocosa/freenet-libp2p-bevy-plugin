use crate::discovery as core_discovery;
use crate::identity;
use crate::ui;
use bevy::prelude::*;

use ui::discovery::{
    BUTTON_LABEL_COLOR, HoveredOccupancyPill, PREVIEW_FRIENDS_MAX, PREVIEW_MEMBER_FONT_SIZE,
    PREVIEW_TITLE_FONT_SIZE, RoomPreviewFriends, SelectedRoom,
};

type ShownFriends = Option<(
    Entity,
    Option<(core_discovery::RoomName, Vec<identity::Username>)>,
)>;

pub fn rebuild_preview_friends(
    mut commands: Commands,
    hovered_pill: Res<HoveredOccupancyPill>,
    selection: Res<SelectedRoom>,
    names: Option<Res<core_discovery::RoomFriendNames>>,
    friends: Option<Single<(Entity, &mut Visibility), With<RoomPreviewFriends>>>,
    mut shown: Local<ShownFriends>,
) {
    let Some(friends) = friends else {
        return;
    };
    let (entity, mut visibility) = friends.into_inner();
    let hovered_pill = hovered_pill.into_inner();
    let selection = selection.into_inner();
    let names = names.map(Res::into_inner);
    let wanted_room = (**hovered_pill)
        .clone()
        .filter(|room| **selection == Some(room.clone()));
    let wanted_names = wanted_room
        .as_ref()
        .and_then(|room| names.as_ref().and_then(|names| names.get(room).cloned()))
        .unwrap_or_default();
    let wanted_entry = wanted_room
        .filter(|_is_the_selected_room| !wanted_names.is_empty())
        .map(|room| (room, wanted_names));
    let wanted_visibility = if wanted_entry.is_some() {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    if *visibility != wanted_visibility {
        *visibility = wanted_visibility;
    }
    let next = Some((entity, wanted_entry.clone()));
    if *shown == next {
        return;
    }
    *shown = next;
    commands.entity(entity).despawn_children();
    let Some((_room, names)) = wanted_entry else {
        return;
    };
    let shown_count = names.len().min(PREVIEW_FRIENDS_MAX);
    let (first_names, remaining_names) = names.split_at(shown_count);
    commands.entity(entity).with_children(|overlay| {
        overlay.spawn((
            Text::new("friends"),
            TextFont {
                font_size: FontSize::Px(PREVIEW_TITLE_FONT_SIZE),
                ..default()
            },
            TextColor(BUTTON_LABEL_COLOR),
        ));
        for name in first_names {
            overlay.spawn((
                Text::new(name.as_str()),
                TextFont {
                    font_size: FontSize::Px(PREVIEW_MEMBER_FONT_SIZE),
                    ..default()
                },
                TextColor(BUTTON_LABEL_COLOR),
            ));
        }
        if !remaining_names.is_empty() {
            overlay.spawn((
                Text::new(format!("and {} more", remaining_names.len())),
                TextFont {
                    font_size: FontSize::Px(PREVIEW_MEMBER_FONT_SIZE),
                    ..default()
                },
                TextColor(BUTTON_LABEL_COLOR),
            ));
        }
    });
}

#[cfg(test)]
mod tests {
    use super::rebuild_preview_friends;
    use crate::discovery as core_discovery;
    use crate::identity;
    use crate::ui;
    use bevy::prelude::*;
    use std::collections::BTreeMap;

    #[test]
    fn rebuild_preview_friends_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("rebuild_preview_friends"),
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
                        ui::discovery::bevy_systems::spawn_preview_friends_card,
                    )
                        .chain(),
                );
                app.add_systems(
                    Update,
                    (
                        ui::discovery::bevy_systems::refresh_room_preview,
                        rebuild_preview_friends,
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
                        world.insert_resource(core_discovery::RoomFriendNames(BTreeMap::from([(
                            core_discovery::RoomName::from("alpha"),
                            vec![
                                identity::Username::from("alice"),
                                identity::Username::from("bob"),
                            ],
                        )])));
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
