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
    use freenet_libp2p_bevy_core::identity;

    fn entry(
        id: &str,
        name: &str,
        status: core_discovery::MemberRoomStatus,
        is_me: bool,
    ) -> core_discovery::RoomRosterEntry {
        core_discovery::RoomRosterEntry {
            id: identity::PeerId::from(id),
            name: String::from(name),
            status,
            is_me,
        }
    }

    fn roster_with(entries: Vec<core_discovery::RoomRosterEntry>) -> core_discovery::RoomRoster {
        core_discovery::RoomRoster {
            own: core_discovery::OwnRoomStatus::Connected,
            count: entries.len(),
            max: 8,
            entries,
        }
    }

    fn app_with(roster_data: core_discovery::RoomRoster, expanded: bool) -> (App, Entity) {
        let mut app = App::new();
        app.insert_resource(roster_data);
        app.init_resource::<ui::roster::MemberIcons>();
        app.insert_resource(ui::roster::ListState(expanded));
        let list = app
            .world_mut()
            .spawn((ui::roster::RosterList, Visibility::Hidden))
            .id();
        app.add_systems(Update, refresh_roster_list);
        (app, list)
    }

    #[test]
    fn test_usage() {
        let roster_data = roster_with(vec![
            entry(
                "far",
                "Far",
                core_discovery::MemberRoomStatus::Connecting,
                false,
            ),
            entry(
                "me",
                "Me",
                core_discovery::MemberRoomStatus::Connected,
                true,
            ),
        ]);
        let (mut app, list) = app_with(roster_data, true);
        app.update();
        let mut rows = app
            .world_mut()
            .query_filtered::<(), With<ui::roster::RosterRow>>();
        assert_eq!(rows.iter(app.world()).count(), 2);
        let mut texts = app.world_mut().query::<(&Text, &ChildOf)>();
        let labels: std::collections::BTreeSet<String> = texts
            .iter(app.world())
            .map(|(text, _)| (**text).clone())
            .collect();
        assert!(labels.contains(&String::from("Far")));
        assert!(labels.contains(&String::from("Me")));
        assert!(labels.contains(&String::from("(you)")));
        assert_eq!(
            *app.world().get::<Visibility>(list).expect("list"),
            Visibility::Visible
        );
    }

    #[test]
    fn a_collapsed_list_stays_hidden_but_rebuilt() {
        let roster_data = roster_with(vec![entry(
            "me",
            "Me",
            core_discovery::MemberRoomStatus::Connected,
            true,
        )]);
        let (mut app, list) = app_with(roster_data, false);
        app.update();
        assert_eq!(
            *app.world().get::<Visibility>(list).expect("list"),
            Visibility::Hidden
        );
        let mut rows = app
            .world_mut()
            .query_filtered::<(), With<ui::roster::RosterRow>>();
        assert_eq!(rows.iter(app.world()).count(), 1);
    }

    #[test]
    fn icons_from_the_hook_fill_the_icon_slot() {
        let roster_data = roster_with(vec![entry(
            "far",
            "Far",
            core_discovery::MemberRoomStatus::Connecting,
            false,
        )]);
        let (mut app, _list) = app_with(roster_data, true);
        app.insert_resource(Assets::<Image>::default());
        let handle = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(icon_sized_image());
        app.world_mut()
            .resource_mut::<ui::roster::MemberIcons>()
            .insert(identity::PeerId::from("far"), handle);
        app.update();
        let mut icon_nodes = app.world_mut().query_filtered::<(), With<ImageNode>>();
        assert_eq!(icon_nodes.iter(app.world()).count(), 1);
    }

    fn icon_sized_image() -> Image {
        Image::new_fill(
            bevy::render::render_resource::Extent3d {
                width: 1,
                height: 1,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            &[255, 255, 255, 255],
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::default(),
        )
    }

    fn preview_roster() -> core_discovery::RoomRoster {
        roster_with(vec![
            entry(
                "12D3KooWcarol0Zm9c",
                "carol",
                core_discovery::MemberRoomStatus::Connecting,
                false,
            ),
            entry(
                "12D3KooWbob000Kd2a",
                "bob",
                core_discovery::MemberRoomStatus::Connected,
                false,
            ),
            entry(
                "me",
                "alice",
                core_discovery::MemberRoomStatus::Connected,
                true,
            ),
        ])
    }

    fn spawn_mock_screen(mut commands: Commands) {
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
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(preview_roster());
        app.init_resource::<ui::roster::MemberIcons>();
        app.init_resource::<ui::roster::ListState>();
        app.add_systems(
            Startup,
            (
                spawn_mock_screen,
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
    }

    const fn preview_noop(_world: &mut World) {}

    fn expand(world: &mut World) {
        world.insert_resource(ui::roster::ListState(true));
    }

    #[test]
    fn refresh_roster_list_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_roster_list"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("collapsed"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("expanded"),
                    apply: expand,
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
