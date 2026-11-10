use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::p2p;
use crate::p2p::{Command, Event};

#[derive(Resource, Deref)]
pub struct CommandSender<Payload: p2p::traits::Message>(pub UnboundedSender<Command<Payload>>);

#[derive(Resource, Deref)]
pub struct EventReceiver<Payload: p2p::traits::Message>(
    pub Mutex<Option<UnboundedReceiver<Event<Payload>>>>,
);

#[derive(Resource, Deref)]
pub struct Libp2pCommandSender(pub UnboundedSender<p2p::Libp2pCommand>);
