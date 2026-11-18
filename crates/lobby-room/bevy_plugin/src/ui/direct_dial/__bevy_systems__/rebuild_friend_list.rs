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
                    | core_direct_dial::FriendPresence::InZone(_) => FRIEND_BADGE_COLOR,
                };
                let where_label = match &friend.presence {
                    core_direct_dial::FriendPresence::Offline => String::from("offline"),
                    core_direct_dial::FriendPresence::Online => String::from("in lobby"),
                    core_direct_dial::FriendPresence::InZone(room) => {
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
    use bevy::prelude::*;

    use super::rebuild_friend_list;
    use freenet_libp2p_bevy_core::direct_dial as core_direct_dial;
    use freenet_libp2p_bevy_core::identity;

    #[test]
    fn rebuild_friend_list_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("rebuild_friend_list"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_direct_dial::FriendsView(vec![
                    core_direct_dial::FriendView {
                        id: identity::PeerId::from("alice"),
                        name: String::from("alice"),
                        presence: core_direct_dial::FriendPresence::Offline,
                    },
                    core_direct_dial::FriendView {
                        id: identity::PeerId::from("bob"),
                        name: String::from("bob"),
                        presence: core_direct_dial::FriendPresence::Online,
                    },
                    core_direct_dial::FriendView {
                        id: identity::PeerId::from("carol"),
                        name: String::from("carol"),
                        presence: core_direct_dial::FriendPresence::InZone(
                            identity::ZoneName::from("room-7"),
                        ),
                    },
                ]));
                app.add_systems(Startup, |mut commands: Commands| {
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
                });
                app.add_systems(Update, rebuild_friend_list);
            },
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("three friends"),
                apply: |_world: &mut World| {},
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
