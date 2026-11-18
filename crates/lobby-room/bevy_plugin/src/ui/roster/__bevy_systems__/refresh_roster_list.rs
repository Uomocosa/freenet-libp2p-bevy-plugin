use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

use ui::roster::{
    ROSTER_CONNECTED_COLOR, ROSTER_CONNECTING_COLOR, ROSTER_DOT_SIZE, ROSTER_NAME_FONT_SIZE,
    ROSTER_ROW_COLOR, ROSTER_ROW_ICON_SIDE, ROSTER_YOU_FONT_SIZE, RosterList, RosterRow,
};

pub fn refresh_roster_list(
    mut commands: Commands,
    roster_data: Option<Res<core_discovery::RoomRoster>>,
    icons: Res<ui::roster::MemberIcons>,
    state: Res<ui::roster::ListState>,
    panel: Single<&mut Visibility, With<RosterList>>,
    list: Single<Entity, With<RosterList>>,
) {
    let is_roster_changed = roster_data.as_ref().is_some_and(Res::is_changed);
    let is_icons_changed = icons.is_changed();
    let wanted_visibility = if **state.into_inner() {
        Visibility::Visible
    } else {
        Visibility::Hidden
    };
    let mut panel = panel.into_inner();
    if *panel != wanted_visibility {
        *panel = wanted_visibility;
    }
    let Some(roster) = roster_data.map(Res::into_inner) else {
        return;
    };
    let icons = icons.into_inner();
    if !is_roster_changed && !is_icons_changed {
        return;
    }
    commands
        .entity(list.into_inner())
        .despawn_children()
        .with_children(|rows| {
            for entry in &roster.entries {
                rows.spawn((
                    RosterRow,
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(ROSTER_ROW_COLOR),
                ))
                .with_children(|row| {
                    row.spawn((
                        Node {
                            width: Val::Px(ROSTER_DOT_SIZE),
                            height: Val::Px(ROSTER_DOT_SIZE),
                            border_radius: BorderRadius::MAX,
                            ..default()
                        },
                        BackgroundColor(match entry.status {
                            core_discovery::MemberRoomStatus::Connected => ROSTER_CONNECTED_COLOR,
                            core_discovery::MemberRoomStatus::Connecting => ROSTER_CONNECTING_COLOR,
                        }),
                    ));
                    if let Some(icon) = icons.get(&entry.id) {
                        row.spawn((
                            ImageNode::new(icon.clone()),
                            Node {
                                width: Val::Px(ROSTER_ROW_ICON_SIDE),
                                height: Val::Px(ROSTER_ROW_ICON_SIDE),
                                ..default()
                            },
                        ));
                    }
                    row.spawn((
                        Text::new(entry.name.clone()),
                        TextFont {
                            font_size: FontSize::Px(ROSTER_NAME_FONT_SIZE),
                            ..default()
                        },
                        TextLayout::no_wrap(),
                    ));
                    if entry.is_me {
                        row.spawn((
                            Text::new("(you)"),
                            TextFont {
                                font_size: FontSize::Px(ROSTER_YOU_FONT_SIZE),
                                ..default()
                            },
                            TextColor(ui::constants::DIM_TEXT_COLOR),
                        ));
                    }
                });
            }
        });
}

#[cfg(test)]
mod tests {
    use super::refresh_roster_list;
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;

    #[test]
    fn refresh_roster_list_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_roster_list"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_discovery::RoomRoster {
                    own: core_discovery::OwnRoomStatus::Connected,
                    count: 3,
                    max: 8,
                    entries: vec![
                        core_discovery::RoomRosterEntry {
                            id: freenet_libp2p_bevy_core::identity::PeerId::from(
                                "12D3KooWcarol0Zm9c",
                            ),
                            name: String::from("carol"),
                            status: core_discovery::MemberRoomStatus::Connecting,
                            is_me: false,
                        },
                        core_discovery::RoomRosterEntry {
                            id: freenet_libp2p_bevy_core::identity::PeerId::from(
                                "12D3KooWbob000Kd2a",
                            ),
                            name: String::from("bob"),
                            status: core_discovery::MemberRoomStatus::Connected,
                            is_me: false,
                        },
                        core_discovery::RoomRosterEntry {
                            id: freenet_libp2p_bevy_core::identity::PeerId::from("me"),
                            name: String::from("alice"),
                            status: core_discovery::MemberRoomStatus::Connected,
                            is_me: true,
                        },
                    ],
                });
                app.init_resource::<ui::roster::MemberIcons>();
                app.init_resource::<ui::roster::ListState>();
                app.add_systems(
                    Startup,
                    (
                        |mut commands: Commands| {
                            let root = commands
                                .spawn(lele_bevy_preview::scene::Scene::centered_root())
                                .id();
                            commands.spawn((
                                ChildOf(root),
                                Node {
                                    width: Val::Percent(100.0),
                                    height: Val::Percent(100.0),
                                    ..default()
                                },
                                BackgroundColor(ui::constants::ROOM_COLOR),
                            ));
                        },
                        ui::roster::bevy_systems::spawn_roster_bubble,
                    ),
                );
                app.add_systems(
                    Update,
                    (
                        ui::roster::bevy_systems::refresh_roster_status,
                        ui::roster::bevy_systems::refresh_roster_tooltips_text,
                        refresh_roster_list,
                    ),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("collapsed"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("expanded"),
                    apply: |world: &mut World| {
                        world.insert_resource(ui::roster::ListState(true));
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
