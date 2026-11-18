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

    fn names(entries: &[(&str, &[&str])]) -> core_discovery::RoomFriendNames {
        core_discovery::RoomFriendNames(
            entries
                .iter()
                .map(|(room, friends)| {
                    (
                        core_discovery::RoomName::from(*room),
                        friends
                            .iter()
                            .map(|friend| identity::Username::from(*friend))
                            .collect(),
                    )
                })
                .collect(),
        )
    }

    fn app_with(
        names: Option<core_discovery::RoomFriendNames>,
        pill: Option<&str>,
        selected: Option<&str>,
    ) -> (App, Entity) {
        let mut app = App::new();
        app.init_resource::<ui::discovery::HoveredOccupancyPill>();
        app.init_resource::<ui::discovery::SelectedRoom>();
        if let Some(names) = names {
            app.insert_resource(names);
        }
        **app
            .world_mut()
            .resource_mut::<ui::discovery::HoveredOccupancyPill>() =
            pill.map(core_discovery::RoomName::from);
        **app
            .world_mut()
            .resource_mut::<ui::discovery::SelectedRoom>() =
            selected.map(core_discovery::RoomName::from);
        let overlay = app
            .world_mut()
            .spawn((ui::discovery::RoomPreviewFriends, Visibility::Hidden))
            .id();
        app.add_systems(Update, rebuild_preview_friends);
        (app, overlay)
    }

    fn visibility(app: &App, overlay: Entity) -> Visibility {
        *app.world()
            .get::<Visibility>(overlay)
            .expect("overlay visibility")
    }

    fn texts(app: &App, overlay: Entity) -> Vec<String> {
        let Some(children) = app.world().get::<Children>(overlay) else {
            return Vec::new();
        };
        children
            .iter()
            .filter_map(|child| app.world().get::<Text>(child))
            .map(|text| text.as_str().to_string())
            .collect()
    }

    #[test]
    fn test_usage() {
        let (mut app, overlay) = app_with(None, None, None);
        app.update();
        assert_eq!(visibility(&app, overlay), Visibility::Hidden);
        assert_eq!(texts(&app, overlay), Vec::<String>::new());
    }

    #[test]
    fn hovering_the_pill_swaps_the_preview_for_the_friend_names() {
        let (mut app, overlay) = app_with(
            Some(names(&[("alpha", &["Alice", "Bob"])])),
            Some("alpha"),
            Some("alpha"),
        );
        app.update();
        assert_eq!(visibility(&app, overlay), Visibility::Inherited);
        assert_eq!(texts(&app, overlay), vec!["friends", "Alice", "Bob"]);
    }

    #[test]
    fn leaving_the_pill_restores_the_preview() {
        let (mut app, overlay) = app_with(
            Some(names(&[("alpha", &["Alice", "Bob"])])),
            Some("alpha"),
            Some("alpha"),
        );
        app.update();
        assert_eq!(visibility(&app, overlay), Visibility::Inherited);
        **app
            .world_mut()
            .resource_mut::<ui::discovery::HoveredOccupancyPill>() = None;
        app.update();
        assert_eq!(visibility(&app, overlay), Visibility::Hidden);
        assert_eq!(texts(&app, overlay), Vec::<String>::new());
    }

    #[test]
    fn a_pill_for_another_room_does_not_reach_this_panel() {
        let (mut app, overlay) = app_with(
            Some(names(&[("alpha", &["Alice"]), ("beta", &["Bob"])])),
            Some("beta"),
            Some("alpha"),
        );
        app.update();
        assert_eq!(visibility(&app, overlay), Visibility::Hidden);
        assert_eq!(texts(&app, overlay), Vec::<String>::new());
    }

    #[test]
    fn a_room_without_friends_keeps_the_preview() {
        let (mut app, overlay) =
            app_with(Some(names(&[("alpha", &[])])), Some("alpha"), Some("alpha"));
        app.update();
        assert_eq!(visibility(&app, overlay), Visibility::Hidden);
        assert_eq!(texts(&app, overlay), Vec::<String>::new());
    }

    #[test]
    fn more_than_the_max_shows_the_remainder_line() {
        let ten_friends = ["a", "b", "c", "d", "e", "f", "g", "h", "i", "j"];
        let (mut app, overlay) = app_with(
            Some(names(&[("alpha", &ten_friends)])),
            Some("alpha"),
            Some("alpha"),
        );
        app.update();
        let texts = texts(&app, overlay);
        assert_eq!(texts.len(), 10);
        assert_eq!(texts.first().map(String::as_str), Some("friends"));
        assert_eq!(texts.last().map(String::as_str), Some("and 2 more"));
        assert_eq!(&texts[1..9], ["a", "b", "c", "d", "e", "f", "g", "h"]);
    }

    fn preview_row(name: &str, occupants: usize) -> core_discovery::LobbyRow {
        core_discovery::LobbyRow {
            name: core_discovery::RoomName::from(name),
            occupants,
            room_size: core_discovery::RoomSize::try_from(8).expect("8"),
            occupancy: core_discovery::RoomOccupancy::HasSpace,
            friends: 2,
        }
    }

    fn preview_build(app: &mut App) {
        app.init_resource::<ui::discovery::HoveredOccupancyPill>();
        app.insert_resource(core_discovery::LobbyRows(vec![preview_row("alpha", 2)]));
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
    }

    const fn preview_noop(_world: &mut World) {}

    fn hover_alpha(world: &mut World) {
        **world.resource_mut::<ui::discovery::HoveredOccupancyPill>() =
            Some(core_discovery::RoomName::from("alpha"));
        world.insert_resource(names(&[("alpha", &["alice", "bob"])]));
    }

    #[test]
    fn rebuild_preview_friends_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("rebuild_preview_friends"),
            kind: lele_bevy_preview::scene::Kind::Plugin,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("preview only"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("friends shown"),
                    apply: hover_alpha,
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
