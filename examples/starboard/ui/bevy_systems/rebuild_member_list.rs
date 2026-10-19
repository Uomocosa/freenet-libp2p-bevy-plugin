use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::discovery;

use crate::starboard;

pub fn rebuild_member_list(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    status: Res<discovery::Status>,
    brushes: Res<starboard::room::RoomBrushes>,
    mut panels: Query<&mut Node, With<starboard::ui::MemberPanel>>,
    list: Single<Entity, With<starboard::ui::MemberList>>,
) {
    let is_status_changed = status.is_changed();
    let is_brushes_changed = brushes.is_changed();
    let status = status.into_inner();
    let brushes = brushes.into_inner();
    let list = list.into_inner();
    let is_shown = starboard::room::is_in_room(status);
    for mut panel in &mut panels {
        let wanted = if is_shown {
            Display::Flex
        } else {
            Display::None
        };
        if panel.display != wanted {
            panel.display = wanted;
        }
    }
    if !is_status_changed && !is_brushes_changed {
        return;
    }
    let Some(room) = status.room.as_ref() else {
        return;
    };
    commands
        .entity(list)
        .despawn_children()
        .with_children(|rows| {
            for peer in room.members.keys() {
                let icon = brushes
                    .get(peer)
                    .copied()
                    .map(|brush| images.add(icon_image(brush)));
                let mut row = rows.spawn((
                    starboard::ui::MemberRow,
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(starboard::ui::ROW_COLOR),
                ));
                row.with_children(|row| {
                    if let Some(icon) = icon {
                        row.spawn((
                            ImageNode::new(icon),
                            Node {
                                width: Val::Px(starboard::ui::ICON_SIDE),
                                height: Val::Px(starboard::ui::ICON_SIDE),
                                ..default()
                            },
                        ));
                    }
                    row.spawn((
                        Text::new(starboard::room::member_name(status, peer)),
                        TextLayout::no_wrap(),
                    ));
                });
            }
        });
}

// needed helper: the solid texture showing a peer's daily shape and colour
fn icon_image(brush: starboard::identity::DailyBrush) -> Image {
    let starboard::render::BrushImage { side, pixels } =
        starboard::render::solid_brush_image(brush.shape, starboard::ui::ICON_RADIUS, brush.color);
    starboard::render::square_image(side, pixels)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::rebuild_member_list;
    use crate::starboard;

    fn room_status(peers: usize) -> discovery::Status {
        let mut members = discovery::Members::new();
        for index in 0..peers {
            members.insert(
                identity::PeerId::from(format!("peer-{index}").as_str()),
                discovery::Member {
                    heartbeat: discovery::Heartbeat {
                        addrs: Vec::new(),
                        updated_at: discovery::UnixTime::from_secs(0),
                        display_name: Some(identity::Username::from(
                            format!("user{index}").as_str(),
                        )),
                    },
                    status: discovery::LinkStatus::Connected,
                },
            );
        }
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(1),
                phase: discovery::RoomPhase::Ready,
                members,
            }),
            ..discovery::Status::default()
        }
    }

    fn room_brushes(status: &discovery::Status) -> starboard::room::RoomBrushes {
        let members = status
            .room
            .as_ref()
            .map(|room| room.members.clone())
            .unwrap_or_default();
        let created_at = status
            .room
            .as_ref()
            .map_or(discovery::UnixTime::from_secs(0), |room| room.created_at);
        let mut brushes = starboard::room::RoomBrushes::default();
        brushes.0 = starboard::identity::room_brushes(
            members.keys(),
            created_at,
            discovery::UnixTime::from_secs(2),
        );
        brushes
    }

    fn preview_build(app: &mut App) {
        let status = room_status(2);
        app.insert_resource(room_brushes(&status));
        app.insert_resource(status);
        app.add_systems(Startup, starboard::ui::bevy_systems::spawn_member_panel)
            .add_systems(Update, rebuild_member_list);
    }

    const fn preview_noop(_world: &mut World) {}

    fn four_peers(world: &mut World) {
        let status = room_status(4);
        let brushes = room_brushes(&status);
        *world.resource_mut::<discovery::Status>() = status;
        *world.resource_mut::<starboard::room::RoomBrushes>() = brushes;
    }

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene() -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("starboard_member_list"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("two peers"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("four peers"),
                    apply: four_peers,
                },
            ],
            timeline: None,
        }
    }

    #[test]
    fn rebuild_member_list_ui_png_preview() {
        lele_bevy_preview::run(&scene(), &config(), env!("CARGO_PKG_NAME")).expect("preview");
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        let status = room_status(2);
        app.insert_resource(room_brushes(&status));
        app.insert_resource(status);
        app.add_systems(Startup, starboard::ui::bevy_systems::spawn_member_panel)
            .add_systems(Update, rebuild_member_list);
        app.update();

        let mut rows = app
            .world_mut()
            .query_filtered::<(), With<starboard::ui::MemberRow>>();
        assert_eq!(rows.iter(app.world()).count(), 2);
    }
}
