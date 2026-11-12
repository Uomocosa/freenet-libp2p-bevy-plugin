use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::ui;

use crate::starboard;

const ICON_RADIUS: u16 = 8;

pub fn refresh_member_icons(
    brushes: Res<starboard::room::RoomBrushes>,
    mut icons: ResMut<ui::roster::MemberIcons>,
    mut images: ResMut<Assets<Image>>,
) {
    let is_brushes_changed = brushes.is_changed();
    let brushes = brushes.into_inner();
    if !is_brushes_changed {
        return;
    }
    icons.clear();
    for (peer, brush) in brushes.iter() {
        icons.insert(peer.clone(), images.add(icon_image(*brush)));
    }
}

// needed helper: the solid texture showing a peer's daily shape and colour
fn icon_image(brush: starboard::identity::DailyBrush) -> Image {
    let starboard::render::BrushImage { side, pixels } =
        starboard::render::solid_brush_image(brush.shape, ICON_RADIUS, brush.color);
    starboard::render::square_image(side, pixels)
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use discovery::state_machine::DiscoveryState;
    use freenet_libp2p_bevy_plugin::discovery;
    use freenet_libp2p_bevy_plugin::{identity, testing, time, ui};

    use super::refresh_member_icons;
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
        let created_at = state
            .room
            .as_ref()
            .map_or(time::UnixTime::EPOCH, |room| room.created_at);
        starboard::room::RoomBrushes(starboard::identity::room_brushes(
            state.occupants().as_slice(),
            created_at,
            time::UnixTime::from_secs(2),
        ))
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
        app.init_resource::<ui::roster::MemberIcons>();
        app.add_systems(Update, refresh_member_icons);
        app.update();
        assert_eq!(app.world().resource::<ui::roster::MemberIcons>().len(), 3);
    }

    #[test]
    fn a_brush_change_replaces_the_icons() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            AssetPlugin::default(),
            ImagePlugin::default_nearest(),
        ));
        let status = room_status(2);
        app.insert_resource(room_brushes(&status));
        app.insert_resource(status);
        app.init_resource::<ui::roster::MemberIcons>();
        app.add_systems(Update, refresh_member_icons);
        app.update();
        let before = app.world().resource::<Assets<Image>>().len();
        *app.world_mut()
            .resource_mut::<starboard::room::RoomBrushes>() = room_brushes(&room_status(3));
        app.update();
        let after = app.world().resource::<Assets<Image>>().len();
        assert_eq!(after, before + 4);
        assert_eq!(app.world().resource::<ui::roster::MemberIcons>().len(), 4);
    }

    fn spawn_board(mut commands: Commands) {
        commands.spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        });
    }

    fn preview_build(app: &mut App) {
        let status = room_status(3);
        app.insert_resource(room_brushes(&status));
        app.insert_resource(status);
        app.insert_resource(discovery::RoomRoster {
            own: discovery::OwnRoomStatus::Connected,
            count: 4,
            max: 8,
            entries: roster_entries(),
        });
        app.add_plugins((
            ui::roster::UiPlugin,
            starboard::app::StarboardTestPlugin::default(),
        ));
        app.add_systems(Startup, (spawn_board, expand_roster_list));
    }

    /// # needed helper: fake roster entries matching the four occupant ids
    fn roster_entries() -> Vec<discovery::RoomRosterEntry> {
        [
            ("peer-0", "user0"),
            ("peer-1", "user1"),
            ("peer-2", "user2"),
        ]
        .into_iter()
        .chain(std::iter::once(("me", "me-name")))
        .map(|(id, name)| discovery::RoomRosterEntry {
            id: identity::PeerId::from(id),
            name: String::from(name),
            status: discovery::MemberRoomStatus::Connected,
            is_me: id == "me",
        })
        .collect()
    }

    fn expand_roster_list(mut commands: Commands) {
        commands.insert_resource(ui::roster::ListState(true));
    }

    const fn preview_noop(_world: &mut World) {}

    #[test]
    fn refresh_member_icons_ui_png_preview() {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::scene::Scene {
            name: String::from("refresh_member_icons"),
            kind: lele_bevy_preview::scene::Kind::System,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("expanded with brush icons"),
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
