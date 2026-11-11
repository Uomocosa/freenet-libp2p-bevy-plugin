use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui_widgets::Button;

use crate::direct_dial;
use crate::discovery;
use direct_dial::{FriendList, FriendRemoveButton, FriendRow};
use discovery::{BUTTON_COLOR, ROOM_COLOR};
use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;

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
                let status = match friend.presence {
                    core_direct_dial::FriendPresence::Offline => "offline",
                    core_direct_dial::FriendPresence::InLobby
                    | core_direct_dial::FriendPresence::InRoom(_) => "connected",
                };
                let label = format!("> {} [{status}]", friend.name);
                parent.spawn((
                    FriendRow(friend.id.clone()),
                    Node {
                        flex_direction: FlexDirection::Row,
                        column_gap: Val::Px(8.0),
                        padding: UiRect::axes(Val::Px(10.0), Val::Px(4.0)),
                        ..default()
                    },
                    BackgroundColor(ROOM_COLOR),
                    children![
                        Text::new(label),
                        (
                            FriendRemoveButton(friend.id.clone()),
                            Button,
                            Hovered::default(),
                            Node {
                                padding: UiRect::axes(Val::Px(8.0), Val::Px(2.0)),
                                ..default()
                            },
                            BackgroundColor(BUTTON_COLOR),
                            children![Text::new("Remove")],
                        )
                    ],
                ));
            }
        });
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use bevy::prelude::*;

    use super::rebuild_friend_list;
    use crate::direct_dial;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::discovery;
    use freenet_libp2p_bevy_core::identity;

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

    fn labels(app: &mut App) -> Vec<String> {
        let mut texts = app
            .world_mut()
            .query_filtered::<&Text, Without<direct_dial::OwnAddressText>>();
        texts
            .iter(app.world())
            .map(|text| text.as_str().to_string())
            .collect()
    }

    fn app_with(view: core_direct_dial::FriendsView) -> App {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.insert_resource(view);
        app.world_mut().spawn(direct_dial::FriendList);
        app.add_systems(Update, rebuild_friend_list);
        app
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(core_direct_dial::FriendsView(vec![
            friend_view("alice", core_direct_dial::FriendPresence::Offline),
            friend_view("bob", core_direct_dial::FriendPresence::InLobby),
        ]));
        app.update();
        let mut rows = app.world_mut().query::<&direct_dial::FriendRow>();
        assert_eq!(rows.iter(app.world()).count(), 2);
    }

    #[test]
    fn rows_follow_the_view() {
        let mut app = app_with(core_direct_dial::FriendsView(vec![friend_view(
            "alice",
            core_direct_dial::FriendPresence::Offline,
        )]));
        app.update();
        assert!(labels(&mut app).iter().any(|text| text.contains("offline")));
        *app.world_mut()
            .resource_mut::<core_direct_dial::FriendsView>() =
            core_direct_dial::FriendsView(vec![friend_view(
                "alice",
                core_direct_dial::FriendPresence::InLobby,
            )]);
        app.update();
        assert!(
            labels(&mut app)
                .iter()
                .any(|text| text.contains("connected"))
        );
    }

    #[test]
    fn the_view_names_and_statuses_drive_the_rows() {
        let view = core_direct_dial::FriendsView::from_friend_ids(
            &[identity::PeerId::from(
                "12D3KooWG6RuxbaapnwtPga2GDrawEZ9wWUsHGYt78dZ6Lrj99RC",
            )],
            &discovery::Lobby::default(),
            &BTreeSet::new(),
        );
        let mut app = app_with(view);
        app.update();
        let shown = labels(&mut app);
        assert!(shown.iter().any(|text| text.contains("12D3Koo...99RC")));
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(preview_view());
        app.add_systems(Startup, spawn_list);
        app.add_systems(Update, rebuild_friend_list);
    }

    fn preview_view() -> core_direct_dial::FriendsView {
        core_direct_dial::FriendsView(vec![friend_view(
            "alice",
            core_direct_dial::FriendPresence::Offline,
        )])
    }

    fn spawn_list(mut commands: Commands) {
        let root = commands
            .spawn(lele_bevy_preview::scene::Scene::centered_root())
            .id();
        commands.spawn((
            ChildOf(root),
            direct_dial::FriendList,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        ));
    }

    const fn preview_noop(_world: &mut World) {}

    fn connect_alice(world: &mut World) {
        *world.resource_mut::<core_direct_dial::FriendsView>() =
            core_direct_dial::FriendsView(vec![friend_view(
                "alice",
                core_direct_dial::FriendPresence::InLobby,
            )]);
    }

    #[test]
    fn rebuild_friend_list_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("rebuild_friend_list"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("one friend offline"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("friend connected"),
                    apply: connect_alice,
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
