use bevy::prelude::*;

use crate::starboard;

pub fn spawn_member_panel(mut commands: Commands) {
    commands.spawn((
        starboard::ui::MemberPanel,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: Val::Px(48.0),
            right: Val::Px(12.0),
            min_width: Val::Px(200.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            padding: UiRect::all(Val::Px(10.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(starboard::ui::PANEL_COLOR),
        children![(
            starboard::ui::MemberList,
            Node {
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                ..default()
            },
        )],
    ));
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::spawn_member_panel;
    use crate::starboard;

    fn room_status(peers: usize) -> discovery::Status {
        let mut members = discovery::Members::new();
        for index in 0..peers {
            members.insert(
                identity::PeerId::from(format!("peer-{index}").as_str()),
                discovery::RoomMember {
                    peer: identity::Peer {
                        id: identity::PeerId::from(format!("peer-{index}").as_str()),
                        addrs: Vec::new(),
                    },
                    updated_at: discovery::UnixTime::from_secs(0),
                    display_name: Some(identity::Username::from(format!("user{index}").as_str())),
                    membership: discovery::RoomMembership::Occupant,
                },
            );
        }
        discovery::Status {
            room: Some(discovery::JoinedRoom {
                name: identity::RoomName::from("alpha"),
                created_at: discovery::UnixTime::from_secs(1),
                phase: discovery::JoinPhase::Ready,
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
        app.add_systems(Startup, spawn_member_panel)
            .add_systems(Update, starboard::ui::bevy_systems::rebuild_member_list);
    }

    const fn preview_noop(_world: &mut World) {}

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene() -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("starboard_member_panel"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![lele_bevy_preview::scene::State {
                label: String::from("room with two peers"),
                apply: preview_noop,
            }],
            timeline: None,
        }
    }

    #[test]
    fn spawn_member_panel_ui_png_preview() {
        lele_bevy_preview::run(&scene(), &config(), env!("CARGO_PKG_NAME")).expect("preview");
    }

    #[test]
    fn test_usage() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        app.add_systems(Startup, spawn_member_panel);
        app.update();
        let mut panels = app
            .world_mut()
            .query_filtered::<(), With<starboard::ui::MemberPanel>>();
        assert_eq!(panels.iter(app.world()).count(), 1);
        let mut lists = app
            .world_mut()
            .query_filtered::<(), With<starboard::ui::MemberList>>();
        assert_eq!(lists.iter(app.world()).count(), 1);
    }
}
