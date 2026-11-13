use crate::ui;
use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
use ui::direct_dial::{DRAWER_ROW_COLOR, FRIEND_OFFLINE_DOT_COLOR};
use ui::direct_dial::{FriendList, FriendRemoveButton, FriendRow, FriendStatusDot};
use ui::discovery::{BUTTON_COLOR, FRIEND_BADGE_COLOR, HOVER_OUTLINE_WIDTH, HoverGlow};

pub fn rebuild_friend_list(
    mut commands: Commands,
    view: Option<Res<core_direct_dial::FriendsView>>,
    list: Single<(Entity, Ref<FriendList>)>,
    mut shown: Local<Vec<core_direct_dial::FriendView>>,
) {
    let wanted: Vec<core_direct_dial::FriendView> =
        view.map(|view| (**view).clone()).unwrap_or_default();
    let (list, marker) = list.into_inner();
    if *shown == wanted && !marker.is_added() {
        return;
    }
    *shown = wanted;
    commands
        .entity(list)
        .despawn_related::<Children>()
        .with_children(|parent| {
            for friend in &*shown {
                let dot = match friend.presence {
                    core_direct_dial::FriendPresence::Offline => FRIEND_OFFLINE_DOT_COLOR,
                    core_direct_dial::FriendPresence::Online
                    | core_direct_dial::FriendPresence::InRoom(_) => FRIEND_BADGE_COLOR,
                };
                let where_label = match &friend.presence {
                    core_direct_dial::FriendPresence::Offline => String::from("offline"),
                    core_direct_dial::FriendPresence::Online => String::from("in lobby"),
                    core_direct_dial::FriendPresence::InRoom(room) => {
                        format!("in {}", room.as_str())
                    }
                };
                parent.spawn((
                    FriendRow(friend.id.clone()),
                    Node {
                        width: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(6.0)),
                        border_radius: BorderRadius::all(Val::Px(8.0)),
                        ..default()
                    },
                    BackgroundColor(DRAWER_ROW_COLOR),
                    children![
                        (
                            FriendStatusDot,
                            Node {
                                width: Val::Px(8.0),
                                height: Val::Px(8.0),
                                border_radius: BorderRadius::MAX,
                                flex_shrink: 0.0,
                                ..default()
                            },
                            BackgroundColor(dot),
                        ),
                        (
                            Node {
                                flex_shrink: 1.0,
                                min_width: Val::ZERO,
                                overflow: Overflow::clip_x(),
                                ..default()
                            },
                            children![Text::new(friend.name.clone())],
                        ),
                        (Node {
                            flex_grow: 1.0,
                            ..default()
                        },),
                        (
                            Text::new(where_label),
                            TextColor(ui::constants::DIM_TEXT_COLOR),
                            TextFont {
                                font_size: FontSize::Px(10.0),
                                ..default()
                            },
                        ),
                        (
                            FriendRemoveButton(friend.id.clone()),
                            Button,
                            Hovered::default(),
                            HoverGlow::default(),
                            UiTransform::default(),
                            Outline::new(Val::Px(HOVER_OUTLINE_WIDTH), Val::ZERO, Color::NONE,),
                            Node {
                                flex_shrink: 0.0,
                                padding: UiRect::axes(Val::Px(6.0), Val::Px(2.0)),
                                border_radius: BorderRadius::all(Val::Px(6.0)),
                                ..default()
                            },
                            BackgroundColor(BUTTON_COLOR),
                            children![Text::new("x")],
                        ),
                    ],
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use crate::ui;
    use std::collections::BTreeSet;

    use bevy::prelude::*;

    use super::rebuild_friend_list;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;
    use freenet_libp2p_bevy_core::presence as core_presence;

    fn friend_view(
        id: &str,
        presence: core_direct_dial::FriendPresence,
    ) -> core_direct_dial::FriendView {
        core_direct_dial::FriendView {
            id: identity::PeerId::from(id),
            name: id.to_string(),
            presence,
        }
    }

    fn app_with(view: core_direct_dial::FriendsView) -> App {
        let mut app = App::new();
        app.insert_resource(view);
        app.world_mut().spawn(ui::direct_dial::FriendList);
        app.add_systems(Update, rebuild_friend_list);
        app
    }

    fn rows(app: &mut App) -> Vec<Entity> {
        let mut rows = app
            .world_mut()
            .query_filtered::<Entity, With<ui::direct_dial::FriendRow>>();
        rows.iter(app.world()).collect()
    }

    fn row_dot_color(app: &mut App, name: &str) -> Color {
        let mut query = app
            .world_mut()
            .query_filtered::<Entity, (With<ui::direct_dial::FriendStatusDot>, With<ChildOf>)>();
        for entity in query.iter(app.world()) {
            let ChildOf(parent) = app.world().get::<ChildOf>(entity).expect("child");
            let row = app
                .world()
                .get::<ui::direct_dial::FriendRow>(*parent)
                .expect("row");
            if row.as_str() == name {
                let BackgroundColor(color) = app
                    .world()
                    .get::<BackgroundColor>(entity)
                    .expect("dot background");
                return *color;
            }
        }
        panic!("no dot for {name}");
    }

    fn texts_of_row(app: &World, entity: Entity) -> Vec<String> {
        fn walk(app: &World, entity: Entity, texts: &mut Vec<String>) {
            if let Some(text) = app.get::<Text>(entity) {
                texts.push(text.as_str().to_string());
            }
            if let Some(children) = app.get::<Children>(entity) {
                for child in children.iter() {
                    walk(app, child, texts);
                }
            }
        }
        let mut texts = Vec::new();
        walk(app, entity, &mut texts);
        texts
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(core_direct_dial::FriendsView(vec![
            friend_view("alice", core_direct_dial::FriendPresence::Offline),
            friend_view("bob", core_direct_dial::FriendPresence::Online),
        ]));
        app.update();
        assert_eq!(rows(&mut app).len(), 2);
    }

    #[test]
    fn rows_follow_the_view() {
        let mut app = app_with(core_direct_dial::FriendsView(vec![friend_view(
            "alice",
            core_direct_dial::FriendPresence::Offline,
        )]));
        app.update();
        assert_eq!(rows(&mut app).len(), 1);
        *app.world_mut()
            .resource_mut::<core_direct_dial::FriendsView>() = core_direct_dial::FriendsView(vec![
            friend_view("alice", core_direct_dial::FriendPresence::Online),
            friend_view(
                "bob",
                core_direct_dial::FriendPresence::InRoom(identity::RoomName::from("room-7")),
            ),
        ]);
        app.update();
        assert_eq!(rows(&mut app).len(), 2);
    }

    #[test]
    fn an_online_friend_has_a_green_dot_and_where_it_is() {
        let mut app = app_with(core_direct_dial::FriendsView(vec![friend_view(
            "bob",
            core_direct_dial::FriendPresence::InRoom(identity::RoomName::from("room-7")),
        )]));
        app.update();
        let row = rows(&mut app)[0];
        assert_eq!(
            row_dot_color(&mut app, "bob"),
            ui::discovery::FRIEND_BADGE_COLOR
        );
        let texts = texts_of_row(app.world(), row);
        assert!(texts.iter().any(|text| text == "in room-7"));
        assert!(texts.iter().any(|text| text == "x"));
    }

    #[test]
    fn a_lobby_friend_says_in_lobby() {
        let mut app = app_with(core_direct_dial::FriendsView(vec![friend_view(
            "bob",
            core_direct_dial::FriendPresence::Online,
        )]));
        app.update();
        let row = rows(&mut app)[0];
        let texts = texts_of_row(app.world(), row);
        assert!(texts.iter().any(|text| text == "in lobby"));
    }

    #[test]
    fn an_offline_friend_has_a_gray_dot() {
        let mut app = app_with(core_direct_dial::FriendsView(vec![friend_view(
            "alice",
            core_direct_dial::FriendPresence::Offline,
        )]));
        app.update();
        assert_eq!(
            row_dot_color(&mut app, "alice"),
            ui::direct_dial::FRIEND_OFFLINE_DOT_COLOR
        );
        let row = rows(&mut app)[0];
        let texts = texts_of_row(app.world(), row);
        assert!(texts.iter().any(|text| text == "offline"));
    }

    #[test]
    fn the_view_names_and_statuses_drive_the_rows() {
        let view = core_direct_dial::FriendsView::from_friend_ids(
            &[identity::PeerId::from(
                "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC",
            )],
            &core_presence::PeerPresence::default(),
            &BTreeSet::new(),
        );
        let mut app = app_with(view);
        app.update();
        let row = rows(&mut app)[0];
        let texts = texts_of_row(app.world(), row);
        assert!(
            texts.iter().any(|text| text == "12D3Koo...99RC"),
            "the short id is the name, not the full peer id"
        );
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(preview_view());
        app.add_systems(Startup, spawn_list);
        app.add_systems(Update, rebuild_friend_list);
    }

    fn preview_view() -> core_direct_dial::FriendsView {
        core_direct_dial::FriendsView(vec![
            friend_view("alice", core_direct_dial::FriendPresence::Offline),
            friend_view("bob", core_direct_dial::FriendPresence::Online),
            friend_view(
                "carol",
                core_direct_dial::FriendPresence::InRoom(identity::RoomName::from("room-7")),
            ),
        ])
    }

    fn spawn_list(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            ui::direct_dial::FriendList,
            Node {
                width: Val::Px(240.0),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ));
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn rebuild_friend_list_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("rebuild_friend_list"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("three friends"),
                apply: preview_noop,
            }],
            timeline: None,
        }
        .run(&lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        })
        .expect("preview");
    }
}
