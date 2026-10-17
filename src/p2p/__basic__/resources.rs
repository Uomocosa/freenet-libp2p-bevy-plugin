use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::p2p;
use crate::p2p::basic::enums::{Command, Event, NetCommand};

#[derive(Resource, Deref)]
pub struct CommandSender<Payload: p2p::Message>(pub UnboundedSender<Command<Payload>>);

#[derive(Resource, Deref)]
pub struct EventReceiver<Payload: p2p::Message>(
    pub Mutex<Option<UnboundedReceiver<Event<Payload>>>>,
);

#[derive(Resource, Deref)]
pub struct NetCommandSender(pub UnboundedSender<NetCommand>);
