use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut};
use tokio::sync::mpsc::UnboundedReceiver;

use crate::discovery;
use crate::identity;
use crate::p2p;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct FreenetPort(pub u16);

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct Libp2pEventTap(pub tokio::sync::broadcast::Receiver<p2p::Libp2pEvent>);

#[derive(Resource, Debug, Deref)]
pub struct LobbyUpdateReceiver(pub Mutex<Option<UnboundedReceiver<discovery::Lobby>>>);

#[derive(Resource, Debug, Deref)]
pub struct FreenetPeersReceiver(pub Mutex<Option<UnboundedReceiver<Vec<identity::FreenetPeer>>>>);

#[derive(Resource, Debug, Deref)]
pub struct HeartbeatTargetSender(
    pub tokio::sync::watch::Sender<Vec<discovery::state_machine::HeartbeatTarget>>,
);
