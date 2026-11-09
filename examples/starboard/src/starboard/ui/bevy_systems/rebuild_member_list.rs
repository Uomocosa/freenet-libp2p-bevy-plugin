use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::discovery;

use crate::starboard;

pub fn rebuild_member_list(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    state: Option<Res<DiscoveryState>>,
    brushes: Res<starboard::room::RoomBrushes>,
    mut panels: Query<&mut Node, With<starboard::ui::MemberPanel>>,
    list: Single<Entity, With<starboard::ui::MemberList>>,
) {
    let is_state_changed = state.as_ref().is_some_and(Res::is_changed);
    let is_brushes_changed = brushes.is_changed();
    let brushes = brushes.into_inner();
    let list = list.into_inner();
    let state = state.map(Res::into_inner);
    let is_shown = state.is_some_and(starboard::room::is_in_room);
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
    if !is_state_changed && !is_brushes_changed {
        return;
    }
    let Some(state) = state else {
        return;
    };
    let Some(room) = state.room.as_ref() else {
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
                        Text::new(starboard::room::member_name(state, peer)),
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
    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::time;
    use freenet_libp2p_bevy_plugin::{discovery, identity, testing};

    use super::rebuild_member_list;
    use crate::starboard;

    fn room_status(peers: usize) -> DiscoveryState {
        let mut members = discovery::Members::new();
        for index in 0..peers {
            members.insert(
                identity::PeerId::from(format!("peer-{index}").as_str()),
                discovery::RoomMember {
                    peer: identity::Peer {
                        id: identity::PeerId::from(format!("peer-{index}").as_str()),
                        addrs: Vec::new(),
                    },
                    updated_at: time::UnixTime::EPOCH,
                    display_name: Some(identity::Username::from(format!("user{index}").as_str())),
                    membership: discovery::RoomMembership::Occupant,
                },
            );
        }
        testing::DiscoveryStateBuilder::new(identity::PeerId::from("me"))
            .in_room(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: time::UnixTime::from_secs(1),
                phase: discovery::JoinPhase::Ready,
                members,
                how_entered: discovery::HowRoomWasEntered::Created,
            })
            .build()
    }

    fn room_brushes(state: &DiscoveryState) -> starboard::room::RoomBrushes {
        let members = state
            .room
            .as_ref()
            .map(|room| room.members.clone())
            .unwrap_or_default();
        let created_at = state
            .room
            .as_ref()
            .map_or(time::UnixTime::EPOCH, |room| room.created_at);
        starboard::room::RoomBrushes(starboard::identity::room_brushes(
            members.keys(),
            created_at,
            time::UnixTime::from_secs(2),
        ))
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
        *world.resource_mut::<DiscoveryState>() = status;
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
        scene().run(&config()).expect("preview");
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
