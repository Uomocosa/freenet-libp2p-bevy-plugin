use std::time::Duration;

use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use freenet_libp2p_bevy_plugin::{discovery, identity, p2p, plugin, room_sync};

use crate::starboard;

pub const SETTLE: Duration = Duration::from_millis(20);
pub const MAX_ROUNDS: usize = 400;

pub fn node_with(game: &identity::GameId, history: Option<room_sync::HistoryConfig>) -> App {
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        ImagePlugin::default_nearest(),
    ));
    app.add_plugins(plugin::P2PPlugin(
        plugin::Config::<starboard::room::CursorUpdate>::new(
            p2p::TransportMode::Both,
            p2p::MdnsMode::Disabled,
        )
        .with_game(game.clone()),
    ));
    app.add_plugins(
        room_sync::SyncPlugin::<starboard::canvas::Canvas>::new(game.clone())
            .with_history_config(history),
    );
    app.insert_resource(room_status(&identity::RoomName::from("alpha"), Vec::new()));
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<AccumulatedMouseScroll>();
    app.add_message::<discovery::Event>();
    app.add_plugins(starboard::app::StarboardPlugin);
    app.update();
    app
}

pub fn room_status(room: &identity::RoomName, members: Vec<identity::PeerId>) -> discovery::Status {
    let mut room_members = discovery::Members::new();
    for peer in members {
        room_members.insert(
            peer.clone(),
            discovery::Member {
                room_member: discovery::RoomMember {
                    peer: identity::Peer {
                        id: peer,
                        addrs: Vec::new(),
                    },
                    updated_at: discovery::UnixTime::from_secs(0),
                    display_name: None,
                    presence: discovery::Presence::InRoom,
                },
                status: discovery::LinkStatus::Connected,
            },
        );
    }
    discovery::Status {
        room: Some(discovery::JoinedRoom {
            name: room.clone(),
            created_at: discovery::UnixTime::from_secs(1),
            phase: discovery::RoomPhase::Ready,
            members: room_members,
        }),
        ..discovery::Status::default()
    }
}

pub fn join(app: &mut App, room: &identity::RoomName, members: Vec<identity::PeerId>) {
    app.insert_resource(room_status(room, members));
    app.world_mut()
        .resource_mut::<Messages<discovery::Event>>()
        .write(discovery::Event::Joined(room.clone()));
}

pub fn dialable(peer: &identity::Peer) -> identity::Peer {
    identity::Peer {
        id: peer.id.clone(),
        addrs: peer
            .addrs
            .iter()
            .map(|addr| identity::PeerAddr(addr.replace("/ip4/0.0.0.0/", "/ip4/127.0.0.1/")))
            .collect(),
    }
}

pub fn ready(app: &App) -> Option<identity::Peer> {
    app.world()
        .resource::<p2p::ReadyChannel>()
        .ready_rx
        .borrow()
        .clone()
}

pub fn wait_ready(app: &mut App) -> identity::Peer {
    for _ in 0..MAX_ROUNDS {
        app.update();
        if let Some(peer) = ready(app) {
            return peer;
        }
        std::thread::sleep(SETTLE);
    }
    panic!("node never became ready");
}

pub fn dial(app: &mut App, target: &identity::Peer) {
    let mut events = app.world().resource::<p2p::NetEventChannel>().subscribe();
    app.world()
        .resource::<p2p::NetCommandSender>()
        .send(p2p::NetCommand::Dial(dialable(target)))
        .expect("dial");
    wait_connected(app, &mut events, &target.id);
}

// needed helper: pump the app until the swarm reports the peer as connected
fn wait_connected(
    app: &mut App,
    events: &mut tokio::sync::broadcast::Receiver<p2p::NetEvent>,
    peer: &identity::PeerId,
) {
    for _ in 0..MAX_ROUNDS {
        app.update();
        while let Ok(event) = events.try_recv() {
            if let p2p::NetEvent::PeerConnected(id) = event
                && &id == peer
            {
                return;
            }
        }
        std::thread::sleep(SETTLE);
    }
    panic!("peer never connected");
}

// no test_usage necessary
