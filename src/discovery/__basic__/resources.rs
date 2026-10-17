use std::sync::Mutex;

use bevy::prelude::{Reflect, ReflectDefault, ReflectResource, Resource};
use derive_more::Deref;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::discovery;
use crate::identity;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Reflect)]
#[reflect(Resource, Default)]
pub struct Snapshot {
    pub lobby: discovery::Lobby,
    pub room: Option<discovery::JoinedRoom>,
    pub freenet_peers: Vec<identity::FreenetPeer>,
    pub lobby_attached: bool,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct FreenetPort(pub u16);

#[derive(Resource, Debug, Deref)]
pub struct CommandSender(pub UnboundedSender<discovery::Command>);

#[derive(Resource, Debug, Deref)]
pub struct EventReceiver(pub Mutex<Option<UnboundedReceiver<discovery::Event>>>);

#[derive(Resource, Debug, Deref)]
pub struct SnapshotReceiver(pub Mutex<Option<UnboundedReceiver<Snapshot>>>);
