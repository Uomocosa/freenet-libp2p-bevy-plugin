//! Status overlay: a top bar with the current room and its peers once joined.
//! Reads `discovery::Status` only; the connecting spinner is the plugin's own screen.
#![allow(clippy::needless_pass_by_value)]
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{discovery, identity};

use crate::shared::status;

const BAR_COLOR: Color = Color::srgb(0.106, 0.122, 0.153);
const DIM_COLOR: Color = Color::srgb(0.62, 0.66, 0.72);
const ROW_COLOR: Color = Color::srgb(0.16, 0.18, 0.23);
const CONNECTED_COLOR: Color = Color::srgb(0.30, 0.82, 0.45);
const LOST_COLOR: Color = Color::srgb(0.45, 0.48, 0.53);
const PEER_SUFFIX_LEN: usize = 6;

pub struct Overlay;

impl Plugin for Overlay {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn)
            .add_systems(Update, (sync_visibility, sync_room_bar));
    }
}

#[derive(Component)]
pub struct RoomBar;

#[derive(Component)]
pub struct PeerCard;

#[derive(Component)]
pub struct RoomView;

#[derive(Component)]
pub struct RoomTitle;

#[derive(Component)]
pub struct PeerCount;

#[derive(Component)]
pub struct PeerList;

fn spawn(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        RoomBar,
        RoomView,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            top: Val::Px(0.0),
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            height: Val::Px(40.0),
            align_items: AlignItems::Center,
            padding: UiRect::axes(Val::Px(12.0), Val::Px(0.0)),
            ..default()
        },
        BackgroundColor(BAR_COLOR),
        children![(RoomTitle, Text::new(""), TextLayout::no_wrap())],
    ));
    commands.spawn((
        PeerCard,
        RoomView,
        Node {
            display: Display::None,
            position_type: PositionType::Absolute,
            bottom: Val::Px(12.0),
            left: Val::Px(12.0),
            min_width: Val::Px(220.0),
            flex_direction: FlexDirection::Column,
            row_gap: Val::Px(6.0),
            padding: UiRect::all(Val::Px(10.0)),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        },
        BackgroundColor(BAR_COLOR),
        children![
            (
                PeerCount,
                Text::new(""),
                TextColor(DIM_COLOR),
                TextLayout::no_wrap(),
            ),
            (
                PeerList,
                Node {
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(4.0),
                    ..default()
                },
            ),
        ],
    ));
}

fn sync_visibility(status: Res<discovery::Status>, mut views: Query<&mut Node, With<RoomView>>) {
    if !status.is_changed() {
        return;
    }
    for mut view in &mut views {
        view.display = display_for(
            status
                .room
                .as_ref()
                .is_some_and(|room| room.phase == discovery::RoomPhase::Ready),
        );
    }
}

fn sync_room_bar(
    mut commands: Commands,
    status: Res<discovery::Status>,
    local_user: Res<status::LocalUser>,
    mut title: Single<&mut Text, (With<RoomTitle>, Without<PeerCount>)>,
    mut count: Single<&mut Text, (With<PeerCount>, Without<RoomTitle>)>,
    list: Single<Entity, With<PeerList>>,
) {
    if !status.is_changed() {
        return;
    }
    let Some(room) = status.room.as_ref() else {
        return;
    };
    let connected = room
        .members
        .values()
        .filter(|member| member.status == discovery::LinkStatus::Connected)
        .count();
    **title = Text::new(format!(
        "Current room: {}   |   you: {}",
        room.name.as_str(),
        local_user.as_str()
    ));
    **count = Text::new(format!(
        "Peers in room ({connected}/{} connected)",
        room.members.len()
    ));
    commands
        .entity(*list)
        .despawn_children()
        .with_children(|rows| {
            for (peer, member) in &room.members {
                let dot = match member.status {
                    discovery::LinkStatus::Connected => CONNECTED_COLOR,
                    discovery::LinkStatus::Lost => LOST_COLOR,
                };
                rows.spawn((
                    Node {
                        align_items: AlignItems::Center,
                        column_gap: Val::Px(8.0),
                        padding: UiRect::axes(Val::Px(8.0), Val::Px(4.0)),
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(ROW_COLOR),
                    children![
                        (
                            Node {
                                width: Val::Px(8.0),
                                height: Val::Px(8.0),
                                border_radius: BorderRadius::MAX,
                                ..default()
                            },
                            BackgroundColor(dot),
                        ),
                        (Text::new(peer_label(peer, member)), TextLayout::no_wrap(),),
                        (
                            Text::new(status_word(member.status)),
                            TextColor(DIM_COLOR),
                            TextLayout::no_wrap(),
                        ),
                    ],
                ));
            }
        });
}

const fn status_word(status: discovery::LinkStatus) -> &'static str {
    match status {
        discovery::LinkStatus::Connected => "connected",
        discovery::LinkStatus::Lost => "lost",
    }
}

const fn display_for(is_shown: bool) -> Display {
    if is_shown {
        Display::Flex
    } else {
        Display::None
    }
}

// needed helper: prefer the peer's chosen name, else a short id
fn peer_label(peer: &identity::PeerId, member: &discovery::Member) -> String {
    member
        .heartbeat
        .display_name
        .as_ref()
        .map_or_else(|| short_peer(peer.as_str()), ToString::to_string)
}

/// # needed helper: libp2p peer ids share a long `12D3KooW` prefix, so the tail identifies them
fn short_peer(full: &str) -> String {
    let start = full.len().saturating_sub(PEER_SUFFIX_LEN);
    full.get(start..)
        .map_or_else(|| full.to_string(), |tail| format!("peer {tail}"))
}

#[cfg(test)]
mod tests {
    use bevy::prelude::*;
    use freenet_libp2p_bevy_plugin::{discovery, identity};

    use super::{Overlay, PeerCard, PeerCount, PeerList, RoomBar, RoomTitle, RoomView, short_peer};

    fn preview_build(app: &mut App) {
        app.insert_resource(discovery::Status::default());
        app.insert_resource(crate::shared::status::LocalUser(identity::Username::from(
            "alice",
        )));
        app.add_plugins(Overlay);
        join_first_room(app.world_mut());
    }

    fn join_first_room(world: &mut World) {
        world.resource_mut::<discovery::Status>().room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("alpha"),
            created_at: discovery::UnixTime::from_secs(0),
            phase: discovery::RoomPhase::Ready,
            members: discovery::Members::new(),
        });
    }

    const fn preview_noop(_world: &mut World) {}

    fn join_room(world: &mut World) {
        let mut status = world.resource_mut::<discovery::Status>();
        status.freenet_peers = vec![identity::FreenetPeer {
            id: identity::FreenetPeerId::from("gateway"),
            addr: std::net::SocketAddr::from(([127, 0, 0, 1], 31337)),
        }];
        let mut members = discovery::Members::new();
        for (peer, name, status) in [
            (
                "12D3KooWbob000Kd2a",
                Some("bob"),
                discovery::LinkStatus::Connected,
            ),
            (
                "12D3KooWcarol0Zm9c",
                Some("carol"),
                discovery::LinkStatus::Connected,
            ),
            ("12D3KooWdave00Qx7f", None, discovery::LinkStatus::Lost),
        ] {
            members.insert(
                identity::PeerId::from(peer),
                discovery::Member {
                    heartbeat: discovery::Heartbeat {
                        addrs: Vec::new(),
                        updated_at: discovery::UnixTime::from_secs(0),
                        display_name: name.map(identity::Username::from),
                    },
                    status,
                },
            );
        }
        status.room = Some(discovery::JoinedRoom {
            name: identity::RoomName::from("gamma"),
            created_at: discovery::UnixTime::from_secs(0),
            phase: discovery::RoomPhase::Ready,
            members,
        });
    }

    fn config() -> lele_bevy_preview::preview::Config {
        let out = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("ui_preview");
        lele_bevy_preview::preview::Config {
            out_dir: out,
            ..Default::default()
        }
    }

    fn scene(
        timeline: Option<lele_bevy_preview::scene::Timeline>,
    ) -> lele_bevy_preview::scene::Scene {
        lele_bevy_preview::scene::Scene {
            name: String::from("overlay"),
            kind: lele_bevy_preview::scene::Kind::App,
            build: preview_build,
            states: vec![
                lele_bevy_preview::scene::State {
                    label: String::from("room without peers"),
                    apply: preview_noop,
                },
                lele_bevy_preview::scene::State {
                    label: String::from("room with peers"),
                    apply: join_room,
                },
            ],
            timeline,
        }
    }

    #[test]
    fn test_usage() {
        assert_eq!(short_peer("12D3KooWabcdefXYZ123"), "peer XYZ123");
        assert_eq!(short_peer("ab"), "peer ab");
    }

    #[test]
    fn overlay_ui_png_preview() {
        let _markers: Option<(RoomBar, PeerCard, RoomView)> = None;
        let _labels: Option<(RoomTitle, PeerCount, PeerList)> = None;
        lele_bevy_preview::run(&scene(None), &config(), env!("CARGO_PKG_NAME")).expect("preview");
    }

    #[test]
    fn overlay_ui_scene_preview() {
        lele_bevy_preview::run(&scene(None), &config(), env!("CARGO_PKG_NAME")).expect("preview");
    }
}
