use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::p2p;
use crate::p2p::basic::enums::{Command, Event, NetCommand};

#[derive(Resource, Deref)]
pub struct CommandChannel<T: p2p::Message>(pub UnboundedSender<Command<T>>);

#[derive(Resource, Deref)]
pub struct EventChannel<T: p2p::Message>(pub Mutex<Option<UnboundedReceiver<Event<T>>>>);

#[derive(Resource, Deref)]
pub struct NetCommandChannel(pub UnboundedSender<NetCommand>);
