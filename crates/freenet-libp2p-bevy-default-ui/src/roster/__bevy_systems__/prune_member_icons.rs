use bevy::prelude::*;

use crate::roster;
use freenet_libp2p_bevy_lobby_room::discovery as core_discovery;

pub fn prune_member_icons(
    roster_data: Option<Res<core_discovery::RoomRoster>>,
    icons: ResMut<roster::MemberIcons>,
) {
    let Some(roster) = roster_data else {
        return;
    };
    if !roster.is_changed() {
        return;
    }
    let icons = icons.into_inner();
    icons.retain(|peer, _| roster.entries.iter().any(|entry| entry.id == *peer));
}

#[cfg(test)]
mod tests {
    use super::prune_member_icons;
    use crate::roster;
    use bevy::prelude::*;
    use freenet_libp2p_bevy_core::identity;
    use freenet_libp2p_bevy_lobby_room::discovery as core_discovery;

    use crate::basic::constants::ROOM_COLOR;

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

    fn app_with(roster_data: core_discovery::RoomRoster) -> App {
        let mut app = App::new();
        app.insert_resource(roster_data);
        app.init_resource::<roster::MemberIcons>();
        app.add_systems(Update, prune_member_icons);
        app
    }

    fn insert_icon(app: &mut App, peer: &str) {
        let handle = app
            .world_mut()
            .resource_mut::<Assets<Image>>()
            .add(pruned_icon_image());
        app.world_mut()
            .resource_mut::<roster::MemberIcons>()
            .insert(identity::PeerId::from(peer), handle);
    }

    #[test]
    fn test_usage() {
        let mut app = app_with(roster_with(vec![
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
        ]));
        app.insert_resource(Assets::<Image>::default());
        insert_icon(&mut app, "far");
        insert_icon(&mut app, "me");
        app.update();
        let icons = app.world().resource::<roster::MemberIcons>();
        assert_eq!(icons.len(), 2);
    }

    #[test]
    fn a_peer_that_leaves_the_roster_loses_its_icon() {
        let mut app = app_with(roster_with(vec![
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
        ]));
        app.insert_resource(Assets::<Image>::default());
        insert_icon(&mut app, "far");
        insert_icon(&mut app, "me");
        app.update();
        *app.world_mut().resource_mut::<core_discovery::RoomRoster>() = roster_with(vec![entry(
            "me",
            "Me",
            core_discovery::MemberRoomStatus::Connected,
            true,
        )]);
        app.update();
        let icons = app.world().resource::<roster::MemberIcons>();
        assert_eq!(icons.len(), 1);
        assert!(!icons.contains_key(&identity::PeerId::from("far")));
    }

    fn pruned_icon_image() -> Image {
        Image::new(
            bevy::render::render_resource::Extent3d {
                width: 17,
                height: 17,
                depth_or_array_layers: 1,
            },
            bevy::render::render_resource::TextureDimension::D2,
            vec![255_u8; 17 * 17 * 4],
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            bevy::asset::RenderAssetUsages::MAIN_WORLD
                | bevy::asset::RenderAssetUsages::RENDER_WORLD,
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

    fn roster_after_a_peer_left() -> core_discovery::RoomRoster {
        roster_with(vec![
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
            BackgroundColor(ROOM_COLOR),
        ));
    }

    fn preview_build(app: &mut App) {
        app.insert_resource(preview_roster());
        app.init_resource::<roster::MemberIcons>();
        app.insert_resource(roster::ListState(true));
        app.add_systems(
            Startup,
            (
                spawn_mock_screen,
                roster::bevy_systems::spawn_roster_bubble,
                insert_hook_icons,
            ),
        );
        app.add_systems(
            Update,
            (
                prune_member_icons,
                roster::bevy_systems::refresh_roster_status,
                roster::bevy_systems::refresh_roster_list,
            )
                .chain(),
        );
    }

    fn peer_leaves(world: &mut World) {
        *world.resource_mut::<core_discovery::RoomRoster>() = roster_after_a_peer_left();
    }

    fn insert_hook_icons(
        mut images: ResMut<Assets<Image>>,
        mut icons: ResMut<roster::MemberIcons>,
    ) {
        for peer in ["12D3KooWcarol0Zm9c", "12D3KooWbob000Kd2a"] {
            let handle = images.add(pruned_icon_image());
            icons.insert(identity::PeerId::from(peer), handle);
        }
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn prune_member_icons_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("prune_member_icons"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("icons from the hook"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("peer left"),
                    apply: peer_leaves,
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
