use crate::discovery as core_discovery;
use crate::ui;
use bevy::prelude::*;

pub fn prune_member_icons(
    roster_data: Option<Res<core_discovery::RoomRoster>>,
    icons: ResMut<ui::roster::MemberIcons>,
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
    use crate::discovery as core_discovery;
    use crate::ui;
    use bevy::prelude::*;
    use core_discovery::{MemberRoomStatus, RoomRosterEntry};
    use freenet_libp2p_bevy_core::identity;

    #[test]
    fn prune_member_icons_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("prune_member_icons"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: |app: &mut App| {
                app.insert_resource(core_discovery::RoomRoster {
                    own: core_discovery::OwnRoomStatus::Connected,
                    count: 3,
                    max: 8,
                    entries: [
                        (
                            "12D3KooWcarol0Zm9c",
                            "carol",
                            MemberRoomStatus::Connecting,
                            false,
                        ),
                        (
                            "12D3KooWbob000Kd2a",
                            "bob",
                            MemberRoomStatus::Connected,
                            false,
                        ),
                        ("me", "alice", MemberRoomStatus::Connected, true),
                    ]
                    .into_iter()
                    .map(|(id, name, status, is_me)| RoomRosterEntry {
                        id: identity::PeerId::from(id),
                        name: String::from(name),
                        status,
                        is_me,
                    })
                    .collect(),
                });
                app.init_resource::<ui::roster::MemberIcons>();
                app.insert_resource(ui::roster::ListState(true));
                let root = app
                    .world_mut()
                    .spawn(lele_bevy_preview::scene::Scene::centered_root())
                    .id();
                app.world_mut().spawn((
                    ChildOf(root),
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(ui::constants::ROOM_COLOR),
                ));
                app.add_systems(Startup, ui::roster::bevy_systems::spawn_roster_bubble);
                app.add_systems(
                    Startup,
                    |mut images: ResMut<Assets<Image>>,
                     mut icons: ResMut<ui::roster::MemberIcons>| {
                        for peer in ["12D3KooWcarol0Zm9c", "12D3KooWbob000Kd2a"] {
                            let handle = images.add(Image::new_fill(
                                bevy::render::render_resource::Extent3d {
                                    width: 17,
                                    height: 17,
                                    depth_or_array_layers: 1,
                                },
                                bevy::render::render_resource::TextureDimension::D2,
                                &[255, 255, 255, 255],
                                bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
                                bevy::asset::RenderAssetUsages::default(),
                            ));
                            icons.insert(identity::PeerId::from(peer), handle);
                        }
                    },
                );
                app.add_systems(
                    Update,
                    (
                        prune_member_icons,
                        ui::roster::bevy_systems::refresh_roster_status,
                        ui::roster::bevy_systems::refresh_roster_list,
                    )
                        .chain(),
                );
            },
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("icons from the hook"),
                    apply: |_world: &mut World| {},
                },
                lele_bevy_preview::scene::State {
                    label: String::from("peer left"),
                    apply: |world: &mut World| {
                        let mut roster = world.resource_mut::<core_discovery::RoomRoster>();
                        roster.entries.retain(|entry| entry.name != "carol");
                        roster.count = roster.entries.len();
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
