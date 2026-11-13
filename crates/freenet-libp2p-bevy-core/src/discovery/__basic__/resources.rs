use std::sync::Mutex;

use bevy::prelude::{Reflect, Resource};
use derive_more::{Deref, DerefMut};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::discovery;
use crate::freenet;
use crate::p2p;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct FreenetPort(pub u16);

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct Libp2pEventTap(pub tokio::sync::broadcast::Receiver<p2p::Libp2pEvent>);

#[derive(Resource, Debug, Deref)]
pub struct LobbyUpdateReceiver(pub Mutex<Option<UnboundedReceiver<discovery::Lobby>>>);

#[derive(Resource, Debug, Deref)]
pub struct FreenetPeersReceiver(pub Mutex<Option<UnboundedReceiver<Vec<freenet::FreenetPeer>>>>);

#[derive(Resource, Debug, Deref)]
pub struct HeartbeatTargetSender(
    pub tokio::sync::watch::Sender<Vec<discovery::state_machine::HeartbeatTarget>>,
);

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref)]
pub struct LobbyRows(pub Vec<discovery::LobbyRow>);

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Reflect)]
pub struct RoomRoster {
    pub own: discovery::OwnRoomStatus,
    pub count: usize,
    pub max: usize,
    pub entries: Vec<discovery::RoomRosterEntry>,
}
