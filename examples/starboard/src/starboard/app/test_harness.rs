use std::time::Duration;

use crate::starboard::app::connection_events::ConnectionEvents;
use bevy::input::mouse::AccumulatedMouseScroll;
use bevy::prelude::*;
use discovery::state_machine::DiscoveryState;
use freenet_libp2p_bevy_plugin::time;
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
    app.add_plugins(discovery::Plugin::new(discovery::Config::default()));
    app.add_plugins(
        room_sync::SyncPlugin::<starboard::canvas::Canvas>::new(game.clone())
            .with_history_config(history),
    );
    app.insert_resource(room_status(
        &identity::PeerId::from("me"),
        &identity::RoomName::from("alpha"),
        Vec::new(),
    ));
    app.init_resource::<ButtonInput<MouseButton>>();
    app.init_resource::<AccumulatedMouseScroll>();
    app.add_message::<discovery::Event>();
    app.add_plugins(starboard::app::StarboardTestPlugin::default());
    app.update();
    app
}

pub fn room_status(
    me: &identity::PeerId,
    room: &identity::RoomName,
    members: Vec<identity::PeerId>,
) -> DiscoveryState {
    let connected = members.clone();
    let mut room_members = discovery::Members::new();
    for peer in members {
        room_members.insert(
            peer.clone(),
            discovery::RoomMember {
                peer: identity::Peer {
                    id: peer,
                    addrs: Vec::new(),
                },
                updated_at: time::UnixTime::EPOCH,
                display_name: None,
                membership: discovery::RoomMembership::Occupant,
            },
        );
    }
    freenet_libp2p_bevy_plugin::lobby_room::testing::DiscoveryStateBuilder::new(me.clone())
        .with_connected(connected)
        .in_room(discovery::JoinedRoom {
            name: room.clone(),
            created_at: time::UnixTime::from_secs(1),
            phase: discovery::JoinPhase::Ready,
            members: room_members,
            how_entered: discovery::HowRoomWasEntered::Created,
        })
        .build()
}

pub fn join(app: &mut App, room: &identity::RoomName, members: Vec<identity::PeerId>) {
    let me = ready(app).map_or_else(|| identity::PeerId::from("me"), |peer| peer.id);
    app.insert_resource(room_status(&me, room, members));
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
    let mut events = ConnectionEvents(
        app.world()
            .resource::<p2p::Libp2pEventChannel>()
            .subscribe(),
    );
    app.world()
        .resource::<p2p::Libp2pCommandSender>()
        .send(p2p::Libp2pCommand::Dial(dialable(target)))
        .expect("dial");
    events.wait_connected(app, &target.id);
}

// no test_usage necessary
