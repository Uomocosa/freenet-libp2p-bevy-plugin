use std::path::PathBuf;

use bevy::prelude::Resource;
use derive_more::{Deref, DerefMut, From};

use crate::direct_dial;
use crate::p2p;

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, From)]
pub struct Config(pub Option<PathBuf>);

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct FriendAddressError(pub Option<String>);

#[derive(Resource, Debug, Default, Clone, PartialEq, Eq, Deref, DerefMut)]
pub struct OwnShareAddress(pub Option<direct_dial::OwnAddress>);

#[derive(Resource, Debug, Deref, DerefMut)]
pub struct DialEventTap(pub tokio::sync::broadcast::Receiver<p2p::Libp2pEvent>);
