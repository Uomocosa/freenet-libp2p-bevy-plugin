use std::sync::Mutex;

use bevy::prelude::Resource;
use derive_more::Deref;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::discovery;

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq, Deref)]
pub struct FreenetPort(pub u16);

#[derive(Resource, Debug, Deref)]
pub struct CommandSender(pub UnboundedSender<discovery::Command>);

#[derive(Resource, Debug, Deref)]
pub struct EventReceiver(pub Mutex<Option<UnboundedReceiver<discovery::Event>>>);

#[derive(Resource, Debug, Deref)]
pub struct StatusReceiver(pub Mutex<Option<UnboundedReceiver<discovery::Status>>>);
