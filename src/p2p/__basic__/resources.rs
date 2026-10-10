use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::p2p;
use crate::p2p::basic::enums::{Command, Event, NetCommand};

#[derive(Resource)]
pub struct Bridge<T: p2p::Message> {
    pub cmd_tx: UnboundedSender<Command<T>>,
    pub event_rx: Mutex<Option<UnboundedReceiver<Event<T>>>>,
}

#[derive(Resource, Deref)]
pub struct NetBridge(pub UnboundedSender<NetCommand>);
