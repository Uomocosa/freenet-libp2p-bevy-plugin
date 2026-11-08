use bevy::prelude::Component;

use crate::peers;

#[derive(Component, Debug, Default, Clone, Copy)]
#[require(peers::KeepAlive)]
pub struct Friend;
